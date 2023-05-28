//===- HWAddressSanitizer.cpp - memory access error detector --------------===//
//
// Part of the LLVM Project, under the Apache License v2.0 with LLVM Exceptions.
// See https://llvm.org/LICENSE.txt for license information.
// SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
//
//===----------------------------------------------------------------------===//
//
/// \file
/// This file is a part of HWAddressSanitizer, an address basic correctness
/// checker based on tagged addressing.
//===----------------------------------------------------------------------===//

#include "llvm/Transforms/Instrumentation/HWAddressSanitizer.h"
#include "llvm/ADT/MapVector.h"
#include "llvm/ADT/STLExtras.h"
#include "llvm/ADT/SmallVector.h"
#include "llvm/ADT/StringExtras.h"
#include "llvm/ADT/StringRef.h"
#include "llvm/Analysis/GlobalsModRef.h"
#include "llvm/Analysis/PostDominators.h"
#include "llvm/Analysis/StackSafetyAnalysis.h"
#include "llvm/Analysis/ValueTracking.h"
#include "llvm/BinaryFormat/Dwarf.h"
#include "llvm/BinaryFormat/ELF.h"
#include "llvm/IR/Attributes.h"
#include "llvm/IR/BasicBlock.h"
#include "llvm/IR/Constant.h"
#include "llvm/IR/Constants.h"
#include "llvm/IR/DataLayout.h"
#include "llvm/IR/DebugInfoMetadata.h"
#include "llvm/IR/DerivedTypes.h"
#include "llvm/IR/Dominators.h"
#include "llvm/IR/Function.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/InlineAsm.h"
#include "llvm/IR/InstIterator.h"
#include "llvm/IR/Instruction.h"
#include "llvm/IR/Instructions.h"
#include "llvm/IR/IntrinsicInst.h"
#include "llvm/IR/Intrinsics.h"
#include "llvm/IR/LLVMContext.h"
#include "llvm/IR/MDBuilder.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/NoFolder.h"
#include "llvm/IR/Type.h"
#include "llvm/IR/Value.h"
#include "llvm/Support/Casting.h"
#include "llvm/Support/CommandLine.h"
#include "llvm/Support/Debug.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/TargetParser/Triple.h"
#include "llvm/Transforms/Instrumentation/AddressSanitizerCommon.h"
#include "llvm/Transforms/Utils/BasicBlockUtils.h"
#include "llvm/Transforms/Utils/MemoryTaggingSupport.h"
#include "llvm/Transforms/Utils/ModuleUtils.h"
#include "llvm/Transforms/Utils/PromoteMemToReg.h"
#include <optional>

using namespace llvm;

#define DEBUG_TYPE "hwasan"

const char kHwasanModuleCtorName[] = "hwasan.module_ctor";
const char kHwasanNoteName[] = "hwasan.note";
const char kHwasanInitName[] = "__hwasan_init";
const char kHwasanPersonalityThunkName[] = "__hwasan_personality_thunk";

const char kHwasanShadowMemoryDynamicAddress[] =
    "__hwasan_shadow_memory_dynamic_address";

// Accesses sizes are powers of two: 1, 2, 4, 8, 16.
static const size_t kNumberOfAccessSizes = 5;

static const size_t kDefaultShadowScale = 4;
static const uint64_t kDynamicShadowSentinel =
    std::numeric_limits<uint64_t>::max();

static const unsigned kShadowBaseAlignment = 32;

static cl::opt<std::string>
    ClMemoryAccessCallbackPrefix("hwasan-memory-access-callback-prefix",
                                 cl::desc("Prefix for memory access callbacks"),
                                 cl::Hidden, cl::init("__hwasan_"));

static cl::opt<bool> ClKasanMemIntrinCallbackPrefix(
    "hwasan-kernel-mem-intrinsic-prefix",
    cl::desc("Use prefix for memory intrinsics in KASAN mode"), cl::Hidden,
    cl::init(false));

static cl::opt<bool> ClInstrumentWithCalls(
    "hwasan-instrument-with-calls",
    cl::desc("instrument reads and writes with callbacks"), cl::Hidden,
    cl::init(false));

static cl::opt<bool> ClInstrumentReads("hwasan-instrument-reads",
                                       cl::desc("instrument read instructions"),
                                       cl::Hidden, cl::init(true));

static cl::opt<bool>
    ClInstrumentWrites("hwasan-instrument-writes",
                       cl::desc("instrument write instructions"), cl::Hidden,
                       cl::init(true));

static cl::opt<bool> ClInstrumentAtomics(
    "hwasan-instrument-atomics",
    cl::desc("instrument atomic instructions (rmw, cmpxchg)"), cl::Hidden,
    cl::init(true));

static cl::opt<bool> ClInstrumentByval("hwasan-instrument-byval",
                                       cl::desc("instrument byval arguments"),
                                       cl::Hidden, cl::init(true));

static cl::opt<bool>
    ClRecover("hwasan-recover",
              cl::desc("Enable recovery mode (continue-after-error)."),
              cl::Hidden, cl::init(false));

static cl::opt<bool> ClInstrumentStack("hwasan-instrument-stack",
                                       cl::desc("instrument stack (allocas)"),
                                       cl::Hidden, cl::init(true));

static cl::opt<bool>
    ClUseStackSafety("hwasan-use-stack-safety", cl::Hidden, cl::init(true),
                     cl::Hidden, cl::desc("Use Stack Safety analysis results"),
                     cl::Optional);

static cl::opt<size_t> ClMaxLifetimes(
    "hwasan-max-lifetimes-for-alloca", cl::Hidden, cl::init(3),
    cl::ReallyHidden,
    cl::desc("How many lifetime ends to handle for a single alloca."),
    cl::Optional);

static cl::opt<bool>
    ClUseAfterScope("hwasan-use-after-scope",
                    cl::desc("detect use after scope within function"),
                    cl::Hidden, cl::init(false));

static cl::opt<bool> ClGenerateTagsWithCalls(
    "hwasan-generate-tags-with-calls",
    cl::desc("generate new tags with runtime library calls"), cl::Hidden,
    cl::init(false));

static cl::opt<bool> ClGlobals("hwasan-globals", cl::desc("Instrument globals"),
                               cl::Hidden, cl::init(false));

static cl::opt<int> ClMatchAllTag(
    "hwasan-match-all-tag",
    cl::desc("don't report bad accesses via pointers with this tag"),
    cl::Hidden, cl::init(-1));

static cl::opt<bool>
    ClEnableKhwasan("hwasan-kernel",
                    cl::desc("Enable KernelHWAddressSanitizer instrumentation"),
                    cl::Hidden, cl::init(false));

// These flags allow to change the shadow mapping and control how shadow memory
// is accessed. The shadow mapping looks like:
//    Shadow = (Mem >> scale) + offset

static cl::opt<uint64_t>
    ClMappingOffset("hwasan-mapping-offset",
                    cl::desc("HWASan shadow mapping offset [EXPERIMENTAL]"),
                    cl::Hidden, cl::init(0));

static cl::opt<bool>
    ClWithIfunc("hwasan-with-ifunc",
                cl::desc("Access dynamic shadow through an ifunc global on "
                         "platforms that support this"),
                cl::Hidden, cl::init(false));

static cl::opt<bool> ClWithTls(
    "hwasan-with-tls",
    cl::desc("Access dynamic shadow through an thread-local pointer on "
             "platforms that support this"),
    cl::Hidden, cl::init(true));

// Mode for selecting how to insert frame record info into the stack ring
// buffer.
enum RecordStackHistoryMode {
  // Do not record frame record info.
  none,

  // Insert instructions into the prologue for storing into the stack ring
  // buffer directly.
  instr,

  // Add a call to __hwasan_add_frame_record in the runtime.
  libcall,
};

static cl::opt<RecordStackHistoryMode> ClRecordStackHistory(
    "hwasan-record-stack-history",
    cl::desc("Record stack frames with tagged allocations in a thread-local "
             "ring buffer"),
    cl::values(clEnumVal(none, "Do not record stack ring history"),
               clEnumVal(instr, "Insert instructions into the prologue for "
                                "storing into the stack ring buffer directly"),
               clEnumVal(libcall, "Add a call to __hwasan_add_frame_record for "
                                  "storing into the stack ring buffer")),
    cl::Hidden, cl::init(instr));

static cl::opt<bool>
    ClInstrumentMemIntrinsics("hwasan-instrument-mem-intrinsics",
                              cl::desc("instrument memory intrinsics"),
                              cl::Hidden, cl::init(true));

static cl::opt<bool>
    ClInstrumentLandingPads("hwasan-instrument-landing-pads",
                            cl::desc("instrument landing pads"), cl::Hidden,
                            cl::init(false));

static cl::opt<bool> ClUseShortGranules(
    "hwasan-use-short-granules",
    cl::desc("use short granules in allocas and outlined checks"), cl::Hidden,
    cl::init(false));

static cl::opt<bool> ClInstrumentPersonalityFunctions(
    "hwasan-instrument-personality-functions",
    cl::desc("instrument personality functions"), cl::Hidden);

static cl::opt<bool> ClInlineAllChecks("hwasan-inline-all-checks",
                                       cl::desc("inline all checks"),
                                       cl::Hidden, cl::init(false));

// Enabled from clang by "-fsanitize-hwaddress-experimental-aliasing".
static cl::opt<bool> ClUsePageAliases("hwasan-experimental-use-page-aliases",
                                      cl::desc("Use page aliasing in HWASan"),
                                      cl::Hidden, cl::init(false));

namespace {

bool shouldUsePageAliases(const Triple &TargetTriple) {
  return ClUsePageAliases && TargetTriple.getArch() == Triple::x86_64;
}

bool shouldInstrumentStack(const Triple &TargetTriple) {
  return !shouldUsePageAliases(TargetTriple) && ClInstrumentStack;
}

bool shouldInstrumentWithCalls(const Triple &TargetTriple) {
  return ClInstrumentWithCalls.getNumOccurrences()
             ? ClInstrumentWithCalls
             : TargetTriple.getArch() == Triple::x86_64;
}

bool mightUseStackSafetyAnalysis(bool DisableOptimization) {
  return ClUseStackSafety.getNumOccurrences() ? ClUseStackSafety
                                              : !DisableOptimization;
}

bool shouldUseStackSafetyAnalysis(const Triple &TargetTriple,
                                  bool DisableOptimization) {
  return shouldInstrumentStack(TargetTriple) &&
         mightUseStackSafetyAnalysis(DisableOptimization);
}

bool shouldDetectUseAfterScope(const Triple &TargetTriple) {
  return ClUseAfterScope && shouldInstrumentStack(TargetTriple);
}

/// An instrumentation pass implementing detection of addressability bugs
/// using tagged pointers.
class HWAddressSanitizer {
public:
  HWAddressSanitizer(Module &M, bool CompileKernel, bool Recover,
                     const StackSafetyGlobalInfo *SSI)
      : M(M), SSI(SSI) {
    this->Recover = ClRecover.getNumOccurrences() > 0 ? ClRecover : Recover;
    this->CompileKernel = ClEnableKhwasan.getNumOccurrences() > 0
                              ? ClEnableKhwasan
                              : CompileKernel;

    initializeModule();
  }

  void setSSI(const StackSafetyGlobalInfo *S) { SSI = S; }

  void sanitizeFunction(Function &F, FunctionAnalysisManager &FAM);
  void initializeModule();
  void createHwasanCtorComdat();

  void initializeCallbacks(Module &M);

  Value *getOpaqueNoopCast(IRBuilder<> &IRB, Value *Val);

  Value *getDynamicShadowIfunc(IRBuilder<> &IRB);
  Value *getShadowNonTls(IRBuilder<> &IRB);

  void untagPointerOperand(Instruction *I, Value *Addr);
  Value *memToShadow(Value *Shadow, IRBuilder<> &IRB);

  int64_t getAccessInfo(bool IsWrite, unsigned AccessSizeIndex);
  void instrumentMemAccessOutline(Value *Ptr, bool IsWrite,
                                  unsigned AccessSizeIndex,
                                  Instruction *InsertBefore);
  void instrumentMemAccessInline(Value *Ptr, bool IsWrite,
                                 unsigned AccessSizeIndex,
                                 Instruction *InsertBefore);
  bool ignoreMemIntrinsic(MemIntrinsic *MI);
  void instrumentMemIntrinsic(MemIntrinsic *MI);
  bool instrumentMemAccess(InterestingMemoryOperand &O);
  bool ignoreAccess(Instruction *Inst, Value *Ptr);
  void getInterestingMemoryOperands(
      Instruction *I, SmallVectorImpl<InterestingMemoryOperand> &Interesting);

  void tagAlloca(IRBuilder<> &IRB, AllocaInst *AI, Value *Tag, size_t Size);
  Value *tagPointer(IRBuilder<> &IRB, Type *Ty, Value *PtrLong, Value *Tag);
  Value *untagPointer(IRBuilder<> &IRB, Value *PtrLong);
  bool instrumentStack(memtag::StackInfo &Info, Value *StackTag, Value *UARTag,
                       const DominatorTree &DT, const PostDominatorTree &PDT,
                       const LoopInfo &LI);
  Value *readRegister(IRBuilder<> &IRB, StringRef Name);
  bool instrumentLandingPads(SmallVectorImpl<Instruction *> &RetVec);
  Value *getNextTagWithCall(IRBuilder<> &IRB);
  Value *getStackBaseTag(IRBuilder<> &IRB);
  Value *getAllocaTag(IRBuilder<> &IRB, Value *StackTag, unsigned AllocaNo);
  Value *getUARTag(IRBuilder<> &IRB);

  Value *getHwasanThreadSlotPtr(IRBuilder<> &IRB, Type *Ty);
  Value *applyTagMask(IRBuilder<> &IRB, Value *OldTag);
  unsigned retagMask(unsigned AllocaNo);

  void emitPrologue(IRBuilder<> &IRB, bool WithFrameRecord);

  void instrumentGlobal(GlobalVariable *GV, uint8_t Tag);
  void instrumentGlobals();

  Value *getPC(IRBuilder<> &IRB);
  Value *getSP(IRBuilder<> &IRB);
  Value *getFrameRecordInfo(IRBuilder<> &IRB);

  void instrumentPersonalityFunctions();

private:
  LLVMContext *C;
  Module &M;
  const StackSafetyGlobalInfo *SSI;
  Triple TargetTriple;
  FunctionCallee HWAsanMemmove, HWAsanMemcpy, HWAsanMemset;
  FunctionCallee HWAsanHandleVfork;

  /// This struct defines the shadow mapping using the rule:
  ///   shadow = (mem >> Scale) + Offset.
  /// If InGlobal is true, then
  ///   extern char __hwasan_shadow[];
  ///   shadow = (mem >> Scale) + &__hwasan_shadow
  /// If InTls is true, then
  ///   extern char *__hwasan_tls;
  ///   shadow = (mem>>Scale) + align_up(__hwasan_shadow, kShadowBaseAlignment)
  ///
  /// If WithFrameRecord is true, then __hwasan_tls will be used to access the
  /// ring buffer for storing stack allocations on targets that support it.
  struct ShadowMapping {
    uint8_t Scale;
    uint64_t Offset;
    bool InGlobal;
    bool InTls;
    bool WithFrameRecord;

    void init(Triple &TargetTriple, bool InstrumentWithCalls);
    Align getObjectAlignment() const { return Align(1ULL << Scale); }
  };

  ShadowMapping Mapping;

  Type *VoidTy = Type::getVoidTy(M.getContext());
  Type *IntptrTy;
  Type *Int8PtrTy;
  Type *Int8Ty;
  Type *Int32Ty;
  Type *Int64Ty = Type::getInt64Ty(M.getContext());

  bool CompileKernel;
  bool Recover;
  bool OutlinedChecks;
  bool UseShortGranules;
  bool InstrumentLandingPads;
  bool InstrumentWithCalls;
  bool InstrumentStack;
  bool DetectUseAfterScope;
  bool UsePageAliases;
  bool UseMatchAllCallback;

  std::optional<uint8_t> MatchAllTag;

  unsigned PointerTagShift;
  uint64_t TagMaskByte;

  Function *HwasanCtorFunction;

  FunctionCallee HwasanMemoryAccessCallback[2][kNumberOfAccessSizes];
  FunctionCallee HwasanMemoryAccessCallbackSized[2];

  FunctionCallee HwasanTagMemoryFunc;
  FunctionCallee HwasanGenerateTagFunc;
  FunctionCallee HwasanRecordFrameRecordFunc;

  Constant *ShadowGlobal;

  Value *ShadowBase = nullptr;
  Value *StackBaseTag = nullptr;
  Value *CachedSP = nullptr;
  GlobalValue *ThreadPtrGlobal = nullptr;
};

} // end anonymous namespace

PreservedAnalyses HWAddressSanitizerPass::run(Module &M,
                                              ModuleAnalysisManager &MAM) {
  const StackSafetyGlobalInfo *SSI = nullptr;
  auto TargetTriple = llvm::Triple(M.getTargetTriple());
  if (shouldUseStackSafetyAnalysis(TargetTriple, Options.DisableOptimization))
    SSI = &MAM.getResult<StackSafetyGlobalAnalysis>(M);

  HWAddressSanitizer HWASan(M, Options.CompileKernel, Options.Recover, SSI);
  auto &FAM = MAM.getResult<FunctionAnalysisManagerModuleProxy>(M).getManager();
  for (Function &F : M)
    HWASan.sanitizeFunction(F, FAM);

  PreservedAnalyses PA = PreservedAnalyses::none();
  // GlobalsAA is considered stateless and does not get invalidated unless
  // explicitly invalidated; PreservedAnalyses::none() is not enough. Sanitizers
  // make changes that require GlobalsAA to be invalidated.
  PA.abandon<GlobalsAA>();
  return PA;
}
void HWAddressSanitizerPass::printPipeline(
    raw_ostream &OS, function_ref<StringRef(StringRef)> MapClassName2PassName) {
  static_cast<PassInfoMixin<HWAddressSanitizerPass> *>(this)->printPipeline(
      OS, MapClassName2PassName);
  OS << '<';
  if (Options.CompileKernel)
    OS << "kernel;";
  if (Options.Recover)
    OS << "recover";
  OS << '>';
}

void HWAddressSanitizer::createHwasanCtorComdat() {
  std::tie(HwasanCtorFunction, std::ignore) =
      getOrCreateSanitizerCtorAndInitFunctions(
          M, kHwasanModuleCtorName, kHwasanInitName,
          /*InitArgTypes=*/{},
          /*InitArgs=*/{},
          // This callback is invoked when the functions are created the first
          // time. Hook them into the global ctors list in that case:
          [&](Function *Ctor, FunctionCallee) {
            Comdat *CtorComdat = M.getOrInsertComdat(kHwasanModuleCtorName);
            Ctor->setComdat(CtorComdat);
            appendToGlobalCtors(M, Ctor, 0, Ctor);
          });

  // Create a note that contains pointers to the list of global
  // descriptors. Adding a note to the output file will cause the linker to
  // create a PT_NOTE program header pointing to the note that we can use to
  // find the descriptor list starting from the program headers. A function
  // provided by the runtime initializes the shadow memory for the globals by
  // accessing the descriptor list via the note. The dynamic loader needs to
  // call this function whenever a library is loaded.
  //
  // The reason why we use a note for this instead of a more conventional
  // approach of having a global constructor pass a descriptor list pointer to
  // the runtime is because of an order of initialization problem. With
  // constructors we can encounter the following problematic scenario:
  //
  // 1) library A depends on library B and also interposes one of B's symbols
  // 2) B's constructors are called before A's (as required for correctness)
  // 3) during construction, B accesses one of its "own" globals (actually
  //    interposed by A) and triggers a HWASAN failure due to the initialization
  //    for A not having happened yet
  //
  // Even without interposition it is possible to run into similar situations in
  // cases where two libraries mutually depend on each other.
  //
  // We only need one note per binary, so put everything for the note in a
  // comdat. This needs to be a comdat with an .init_array section to prevent
  // newer versions of lld from discarding the note.
  //
  // Create the note even if we aren't instrumenting globals. This ensures that
  // binaries linked from object files with both instrumented and
  // non-instrumented globals will end up with a note, even if a comdat from an
  // object file with non-instrumented globals is selected. The note is harmless
  // if the runtime doesn't support it, since it will just be ignored.
  Comdat *NoteComdat = M.getOrInsertComdat(kHwasanModuleCtorName);

  Type *Int8Arr0Ty = ArrayType::get(Int8Ty, 0);
  auto *Start =
      new GlobalVariable(M, Int8Arr0Ty, true, GlobalVariable::ExternalLinkage,
                         nullptr, "__start_hwasan_globals");
  Start->setVisibility(GlobalValue::HiddenVisibility);
  auto *Stop =
      new GlobalVariable(M, Int8Arr0Ty, true, GlobalVariable::ExternalLinkage,
                         nullptr, "__stop_hwasan_globals");
  Stop->setVisibility(GlobalValue::HiddenVisibility);

  // Null-terminated so actually 8 bytes, which are required in order to align
  // the note properly.
  auto *Name = ConstantDataArray::get(*C, "LLVM\0\0\0");

  auto *NoteTy = StructType::get(Int32Ty, Int32Ty, Int32Ty, Name->getType(),
                                 Int32Ty, Int32Ty);
  auto *Note =
      new GlobalVariable(M, NoteTy, /*isConstant=*/true,
                         GlobalValue::PrivateLinkage, nullptr, kHwasanNoteName);
  Note->setSection(".note.hwasan.globals");
  Note->setComdat(NoteComdat);
  Note->setAlignment(Align(4));

  // The pointers in the note need to be relative so that the note ends up being
  // placed in rodata, which is the standard location for notes.
  auto CreateRelPtr = [&](Constant *Ptr) {
    return ConstantExpr::getTrunc(
        ConstantExpr::getSub(ConstantExpr::getPtrToInt(Ptr, Int64Ty),
                             ConstantExpr::getPtrToInt(Note, Int64Ty)),
        Int32Ty);
  };
  Note->setInitializer(ConstantStruct::getAnon(
      {ConstantInt::get(Int32Ty, 8),                           // n_namesz
       ConstantInt::get(Int32Ty, 8),                           // n_descsz
       ConstantInt::get(Int32Ty, ELF::NT_LLVM_HWASAN_GLOBALS), // n_type
       Name, CreateRelPtr(Start), CreateRelPtr(Stop)}));
  appendToCompilerUsed(M, Note);

  // Create a zero-length global in hwasan_globals so that the linker will
  // always create start and stop symbols.
  auto *Dummy = new GlobalVariable(
      M, Int8Arr0Ty, /*isConstantGlobal*/ true, GlobalVariable::PrivateLinkage,
      Constant::getNullValue(Int8Arr0Ty), "hwasan.dummy.global");
  Dummy->setSection("hwasan_globals");
  Dummy->setComdat(NoteComdat);
  Dummy->setMetadata(LLVMContext::MD_associated,
                     MDNode::get(*C, ValueAsMetadata::get(Note)));
  appendToCompilerUsed(M, Dummy);
}

/// Module-level initialization.
///
/// inserts a call to __hwasan_init to the module's constructor list.
void HWAddressSanitizer::initializeModule() {
  LLVM_DEBUG(dbgs() << "Init " << M.getName() << "\n");
  auto &DL = M.getDataLayout();

  TargetTriple = Triple(M.getTargetTriple());

  // x86_64 currently has two modes:
  // - Intel LAM (default)
  // - pointer aliasing (heap only)
  bool IsX86_64 = TargetTriple.getArch() == Triple::x86_64;
  UsePageAliases = shouldUsePageAliases(TargetTriple);
  InstrumentWithCalls = shouldInstrumentWithCalls(TargetTriple);
  InstrumentStack = shouldInstrumentStack(TargetTriple);
  DetectUseAfterScope = shouldDetectUseAfterScope(TargetTriple);
  PointerTagShift = IsX86_64 ? 57 : 56;
  TagMaskByte = IsX86_64 ? 0x3F : 0xFF;

  Mapping.init(TargetTriple, InstrumentWithCalls);

  C = &(M.getContext());
  IRBuilder<> IRB(*C);
  IntptrTy = IRB.getIntPtrTy(DL);
  Int8PtrTy = IRB.getInt8PtrTy();
  Int8Ty = IRB.getInt8Ty();
  Int32Ty = IRB.getInt32Ty();

  HwasanCtorFunction = nullptr;

  // Older versions of Android do not have the required runtime support for
  // short granules, global or personality function instrumentation. On other
  // platforms we currently require using the latest version of the runtime.
  bool NewRuntime =
      !TargetTriple.isAndroid() || !TargetTriple.isAndroidVersionLT(30);

  UseShortGranules =
      ClUseShortGranules.getNumOccurrences() ? ClUseShortGranules : NewRuntime;
  OutlinedChecks =
      (TargetTriple.isAArch64() || TargetTriple.isRISCV64()) &&
      TargetTriple.isOSBinFormatELF() &&
      (ClInlineAllChecks.getNumOccurrences() ? !ClInlineAllChecks : !Recover);

  if (ClMatchAllTag.getNumOccurrences()) {
    if (ClMatchAllTag != -1) {
      MatchAllTag = ClMatchAllTag & 0xFF;
    }
  } else if (CompileKernel) {
    MatchAllTag = 0xFF;
  }
  UseMatchAllCallback = !CompileKernel && MatchAllTag.has_value();

  // If we don't have personality function support, fall back to landing pads.
  InstrumentLandingPads = ClInstrumentLandingPads.getNumOccurrences()
                              ? ClInstrumentLandingPads
                              : !NewRuntime;

  if (!CompileKernel) {
    createHwasanCtorComdat();
    bool InstrumentGlobals =
        ClGlobals.getNumOccurrences() ? ClGlobals : NewRuntime;

    if (InstrumentGlobals && !UsePageAliases)
      instrumentGlobals();

    bool InstrumentPersonalityFunctions =
        ClInstrumentPersonalityFunctions.getNumOccurrences()
            ? ClInstrumentPersonalityFunctions
            : NewRuntime;
    if (InstrumentPersonalityFunctions)
      instrumentPersonalityFunctions();
  }

  if (!TargetTriple.isAndroid()) {
    Constant *C = M.getOrInsertGlobal("__hwasan_tls", IntptrTy, [&] {
      auto *GV = new GlobalVariable(M, IntptrTy, /*isConstant=*/false,
                                    GlobalValue::ExternalLinkage, nullptr,
                                    "__hwasan_tls", nullptr,
                                    GlobalVariable::InitialExecTLSModel);
      appendToCompilerUsed(M, GV);
      return GV;
    });
    ThreadPtrGlobal = cast<GlobalVariable>(C);
  }
}

void HWAddressSanitizer::initializeCallbacks(Module &M) {
  IRBuilder<> IRB(*C);
  const std::string MatchAllStr = UseMatchAllCallback ? "_match_all" : "";
  FunctionType *HwasanMemoryAccessCallbackSizedFnTy,
      *HwasanMemoryAccessCallbackFnTy, *HWAsanMemTransferFnTy,
      *HWAsanMemsetFnTy;
  if (UseMatchAllCallback) {
    HwasanMemoryAccessCallbackSizedFnTy =
        FunctionType::get(VoidTy, {IntptrTy, IntptrTy, Int8Ty}, false);
    HwasanMemoryAccessCallbackFnTy =
        FunctionType::get(VoidTy, {IntptrTy, Int8Ty}, false);
    HWAsanMemTransferFnTy = FunctionType::get(
        Int8PtrTy, {Int8PtrTy, Int8PtrTy, IntptrTy, Int8Ty}, false);
    HWAsanMemsetFnTy = FunctionType::get(
        Int8PtrTy, {Int8PtrTy, Int32Ty, IntptrTy, Int8Ty}, false);
  } else {
    HwasanMemoryAccessCallbackSizedFnTy =
        FunctionType::get(VoidTy, {IntptrTy, IntptrTy}, false);
    HwasanMemoryAccessCallbackFnTy =
        FunctionType::get(VoidTy, {IntptrTy}, false);
    HWAsanMemTransferFnTy =
        FunctionType::get(Int8PtrTy, {Int8PtrTy, Int8PtrTy, IntptrTy}, false);
    HWAsanMemsetFnTy =
        FunctionType::get(Int8PtrTy, {Int8PtrTy, Int32Ty, IntptrTy}, false);
  }

  for (size_t AccessIsWrite = 0; AccessIsWrite <= 1; AccessIsWrite++) {
    const std::string TypeStr = AccessIsWrite ? "store" : "load";
    const std::string EndingStr = Recover ? "_noabort" : "";

    HwasanMemoryAccessCallbackSized[AccessIsWrite] = M.getOrInsertFunction(
        ClMemoryAccessCallbackPrefix + TypeStr + "N" + MatchAllStr + EndingStr,
        HwasanMemoryAccessCallbackSizedFnTy);

    for (size_t AccessSizeIndex = 0; AccessSizeIndex < kNumberOfAccessSizes;
         AccessSizeIndex++) {
      HwasanMemoryAccessCallback[AccessIsWrite][AccessSizeIndex] =
          M.getOrInsertFunction(ClMemoryAccessCallbackPrefix + TypeStr +
                                    itostr(1ULL << AccessSizeIndex) +
                                    MatchAllStr + EndingStr,
                                HwasanMemoryAccessCallbackFnTy);
    }
  }

  const std::string MemIntrinCallbackPrefix =
      (CompileKernel && !ClKasanMemIntrinCallbackPrefix)
          ? std::string("")
          : ClMemoryAccessCallbackPrefix;

  HWAsanMemmove = M.getOrInsertFunction(
      MemIntrinCallbackPrefix + "memmove" + MatchAllStr, HWAsanMemTransferFnTy);
  HWAsanMemcpy = M.getOrInsertFunction(
      MemIntrinCallbackPrefix + "memcpy" + MatchAllStr, HWAsanMemTransferFnTy);
  HWAsanMemset = M.getOrInsertFunction(
      MemIntrinCallbackPrefix + "memset" + MatchAllStr, HWAsanMemsetFnTy);

  HwasanTagMemoryFunc = M.getOrInsertFunction("__hwasan_tag_memory", VoidTy,
                                              Int8PtrTy, Int8Ty, IntptrTy);
  HwasanGenerateTagFunc =
      M.getOrInsertFunction("__hwasan_generate_tag", Int8Ty);

  HwasanRecordFrameRecordFunc =
      M.getOrInsertFunction("__hwasan_add_frame_record", VoidTy, Int64Ty);

  ShadowGlobal =
      M.getOrInsertGlobal("__hwasan_shadow", ArrayType::get(Int8Ty, 0));

  HWAsanHandleVfork =
      M.getOrInsertFunction("__hwasan_handle_vfork", VoidTy, IntptrTy);
}

Value *HWAddressSanitizer::getOpaqueNoopCast(IRBuilder<> &IRB, Value *Val) {
  // An empty inline asm with input reg == output reg.
  // An opaque no-op cast, basically.
  // This prevents code bloat as a result of rematerializing trivial definitions
  // such as constants or global addresses at every load and store.
  InlineAsm *Asm =
      InlineAsm::get(FunctionType::get(Int8PtrTy, {Val->getType()}, false),
                     StringRef(""), StringRef("=r,0"),
                     /*hasSideEffects=*/false);
  return IRB.CreateCall(Asm, {Val}, ".hwasan.shadow");
}

Value *HWAddressSanitizer::getDynamicShadowIfunc(IRBuilder<> &IRB) {
  return getOpaqueNoopCast(IRB, ShadowGlobal);
}

Value *HWAddressSanitizer::getShadowNonTls(IRBuilder<> &IRB) {
  if (Mapping.Offset != kDynamicShadowSentinel)
    return getOpaqueNoopCast(
        IRB, ConstantExpr::getIntToPtr(
                 ConstantInt::get(IntptrTy, Mapping.Offset), Int8PtrTy));

  if (Mapping.InGlobal)
    return getDynamicShadowIfunc(IRB);

  Value *GlobalDynamicAddress =
      IRB.GetInsertBlock()->getParent()->getParent()->getOrInsertGlobal(
          kHwasanShadowMemoryDynamicAddress, Int8PtrTy);
  return IRB.CreateLoad(Int8PtrTy, GlobalDynamicAddress);
}

bool HWAddressSanitizer::ignoreAccess(Instruction *Inst, Value *Ptr) {
  // Do not instrument accesses from different address spaces; we cannot deal
  // with them.
  Type *PtrTy = cast<PointerType>(Ptr->getType()->getScalarType());
  if (PtrTy->getPointerAddressSpace() != 0)
    return true;

  // Ignore swifterror addresses.
  // swifterror memory addresses are mem2reg promoted by instruction
  // selection. As such they cannot have regular uses like an instrumentation
  // function and it makes no sense to track them as memory.
  if (Ptr->isSwiftError())
    return true;

  if (findAllocaForValue(Ptr)) {
    if (!InstrumentStack)
      return true;
    if (SSI && SSI->stackAccessIsSafe(*Inst))
      return true;
  }
  return false;
}

void HWAddressSanitizer::getInterestingMemoryOperands(
    Instruction *I, SmallVectorImpl<InterestingMemoryOperand> &Interesting) {
  // Skip memory accesses inserted by another instrumentation.
  if (I->hasMetadata(LLVMContext::MD_nosanitize))
    return;

  // Do not instrument the load fetching the dynamic shadow address.
  if (ShadowBase == I)
    return;

  if (LoadInst *LI = dyn_cast<LoadInst>(I)) {
    if (!ClInstrumentReads || ignoreAccess(I, LI->getPointerOperand()))
      return;
    Interesting.emplace_back(I, LI->getPointerOperandIndex(), false,
                             LI->getType(), LI->getAlign());
  } else if (StoreInst *SI = dyn_cast<StoreInst>(I)) {
    if (!ClInstrumentWrites || ignoreAccess(I, SI->getPointerOperand()))
      return;
    Interesting.emplace_back(I, SI->getPointerOperandIndex(), true,
                             SI->getValueOperand()->getType(), SI->getAlign());
  } else if (AtomicRMWInst *RMW = dyn_cast<AtomicRMWInst>(I)) {
    if (!ClInstrumentAtomics || ignoreAccess(I, RMW->getPointerOperand()))
      return;
    Interesting.emplace_back(I, RMW->getPointerOperandIndex(), true,
                             RMW->getValOperand()->getType(), std::nullopt);
  } else if (AtomicCmpXchgInst *XCHG = dyn_cast<AtomicCmpXchgInst>(I)) {
    if (!ClInstrumentAtomics || ignoreAccess(I, XCHG->getPointerOperand()))
      return;
    Interesting.emplace_back(I, XCHG->getPointerOperandIndex(), true,
                             XCHG->getCompareOperand()->getType(),
                             std::nullopt);
  } else if (auto *CI = dyn_cast<CallInst>(I)) {
    for (unsigned ArgNo = 0; ArgNo < CI->arg_size(); ArgNo++) {
      if (!ClInstrumentByval || !CI->isByValArgument(ArgNo) ||
          ignoreAccess(I, CI->getArgOperand(ArgNo)))
        continue;
      Type *Ty = CI->getParamByValType(ArgNo);
      Interesting.emplace_back(I, ArgNo, false, Ty, Align(1));
    }
  }
}

static unsigned getPointerOperandIndex(Instruction *I) {
  if (LoadInst *LI = dyn_cast<LoadInst>(I))
    return LI->getPointerOperandIndex();
  if (StoreInst *SI = dyn_cast<StoreInst>(I))
    return SI->getPointerOperandIndex();
  if (AtomicRMWInst *RMW = dyn_cast<AtomicRMWInst>(I))
    return RMW->getPointerOperandIndex();
  if (AtomicCmpXchgInst *XCHG = dyn_cast<AtomicCmpXchgInst>(I))
    return XCHG->getPointerOperandIndex();
  report_fatal_error("Unexpected instruction");
  return -1;
}

static size_t TypeSizeToSizeIndex(uint32_t TypeSize) {
  size_t Res = llvm::countr_zero(TypeSize / 8);
  assert(Res < kNumberOfAccessSizes);
  return Res;
}

void HWAddressSanitizer::untagPointerOperand(Instruction *I, Value *Addr) {
  if (TargetTriple.isAArch64() || TargetTriple.getArch() == Triple::x86_64 ||
      TargetTriple.isRISCV64())
    return;

  IRBuilder<> IRB(I);
  Value *AddrLong = IRB.CreatePointerCast(Addr, IntptrTy);
  Value *UntaggedPtr =
      IRB.CreateIntToPtr(untagPointer(IRB, AddrLong), Addr->getType());
  I->setOperand(getPointerOperandIndex(I), UntaggedPtr);
}

Value *HWAddressSanitizer::memToShadow(Value *Mem, IRBuilder<> &IRB) {
  // Mem >> Scale
  Value *Shadow = IRB.CreateLShr(Mem, Mapping.Scale);
  if (Mapping.Offset == 0)
    return IRB.CreateIntToPtr(Shadow, Int8PtrTy);
  // (Mem >> Scale) + Offset
  return IRB.CreateGEP(Int8Ty, ShadowBase, Shadow);
}

int64_t HWAddressSanitizer::getAccessInfo(bool IsWrite,
                                          unsigned AccessSizeIndex) {
  return (CompileKernel << HWASanAccessInfo::CompileKernelShift) |
         (MatchAllTag.has_value() << HWASanAccessInfo::HasMatchAllShift) |
         (MatchAllTag.value_or(0) << HWASanAccessInfo::MatchAllShift) |
         (Recover << HWASanAccessInfo::RecoverShift) |
         (IsWrite << HWASanAccessInfo::IsWriteShift) |
         (AccessSizeIndex << HWASanAccessInfo::AccessSizeShift);
}

void HWAddressSanitizer::instrumentMemAccessOutline(Value *Ptr, bool IsWrite,
                                                    unsigned AccessSizeIndex,
                                                    Instruction *InsertBefore) {
  assert(!UsePageAliases);
  const int64_t AccessInfo = getAccessInfo(IsWrite, AccessSizeIndex);
  IRBuilder<> IRB(InsertBefore);
  Module *M = IRB.GetInsertBlock()->getParent()->getParent();
  Ptr = IRB.CreateBitCast(Ptr, Int8PtrTy);
  IRB.CreateCall(Intrinsic::getDeclaration(
                     M, UseShortGranules
                            ? Intrinsic::hwasan_check_memaccess_shortgranules
                            : Intrinsic::hwasan_check_memaccess),
                 {ShadowBase, Ptr, ConstantInt::get(Int32Ty, AccessInfo)});
}

void HWAddressSanitizer::instrumentMemAccessInline(Value *Ptr, bool IsWrite,
                                                   unsigned AccessSizeIndex,
                                                   Instruction *InsertBefore) {
  assert(!UsePageAliases);
  const int64_t AccessInfo = getAccessInfo(IsWrite, AccessSizeIndex);
  IRBuilder<> IRB(InsertBefore);

  Value *PtrLong = IRB.CreatePointerCast(Ptr, IntptrTy);
  Value *PtrTag =
      IRB.CreateTrunc(IRB.CreateLShr(PtrLong, PointerTagShift), Int8Ty);
  Value *AddrLong = untagPointer(IRB, PtrLong);
  Value *Shadow = memToShadow(AddrLong, IRB);
  Value *MemTag = IRB.CreateLoad(Int8Ty, Shadow);
  Value *TagMismatch = IRB.CreateICmpNE(PtrTag, MemTag);

  if (MatchAllTag.has_value()) {
    Value *TagNotIgnored = IRB.CreateICmpNE(
        PtrTag, ConstantInt::get(PtrTag->getType(), *MatchAllTag));
    TagMismatch = IRB.CreateAnd(TagMismatch, TagNotIgnored);
  }

  Instruction *CheckTerm =
      SplitBlockAndInsertIfThen(TagMismatch, InsertBefore, false,
                                MDBuilder(*C).createBranchWeights(1, 100000));

  IRB.SetInsertPoint(CheckTerm);
  Value *OutOfShortGranuleTagRange =
      IRB.CreateICmpUGT(MemTag, ConstantInt::get(Int8Ty, 15));
  Instruction *CheckFailTerm =
      SplitBlockAndInsertIfThen(OutOfShortGranuleTagRange, CheckTerm, !Recover,
                                MDBuilder(*C).createBranchWeights(1, 100000));

  IRB.SetInsertPoint(CheckTerm);
  Value *PtrLowBits = IRB.CreateTrunc(IRB.CreateAnd(PtrLong, 15), Int8Ty);
  PtrLowBits = IRB.CreateAdd(
      PtrLowBits, ConstantInt::get(Int8Ty, (1 << AccessSizeIndex) - 1));
  Value *PtrLowBitsOOB = IRB.CreateICmpUGE(PtrLowBits, MemTag);
  SplitBlockAndInsertIfThen(PtrLowBitsOOB, CheckTerm, false,
                            MDBuilder(*C).createBranchWeights(1, 100000),
                            (DomTreeUpdater *)nullptr, nullptr,
                            CheckFailTerm->getParent());

  IRB.SetInsertPoint(CheckTerm);
  Value *InlineTagAddr = IRB.CreateOr(AddrLong, 15);
  InlineTagAddr = IRB.CreateIntToPtr(InlineTagAddr, Int8PtrTy);
  Value *InlineTag = IRB.CreateLoad(Int8Ty, InlineTagAddr);
  Value *InlineTagMismatch = IRB.CreateICmpNE(PtrTag, InlineTag);
  SplitBlockAndInsertIfThen(InlineTagMismatch, CheckTerm, false,
                            MDBuilder(*C).createBranchWeights(1, 100000),
                            (DomTreeUpdater *)nullptr, nullptr,
                            CheckFailTerm->getParent());

  IRB.SetInsertPoint(CheckFailTerm);
  InlineAsm *Asm;
  switch (TargetTriple.getArch()) {
  case Triple::x86_64:
    // The signal handler will find the data address in rdi.
    Asm = InlineAsm::get(
        FunctionType::get(VoidTy, {PtrLong->getType()}, false),
        "int3\nnopl " +
            itostr(0x40 + (AccessInfo & HWASanAccessInfo::RuntimeMask)) +
            "(%rax)",
        "{rdi}",
        /*hasSideEffects=*/true);
    break;
  case Triple::aarch64:
  case Triple::aarch64_be:
    // The signal handler will find the data address in x0.
    Asm = InlineAsm::get(
        FunctionType::get(VoidTy, {PtrLong->getType()}, false),
        "brk #" + itostr(0x900 + (AccessInfo & HWASanAccessInfo::RuntimeMask)),
        "{x0}",
        /*hasSideEffects=*/true);
    break;
  case Triple::riscv64:
    // The signal handler will find the data address in x10.
    Asm = InlineAsm::get(
        FunctionType::get(VoidTy, {PtrLong->getType()}, false),
        "ebreak\naddiw x0, x11, " +
            itostr(0x40 + (AccessInfo & HWASanAccessInfo::RuntimeMask)),
        "{x10}",
        /*hasSideEffects=*/true);
    break;
  default:
    report_fatal_error("unsupported architecture");
  }
  IRB.CreateCall(Asm, PtrLong);
  if (Recover)
    cast<BranchInst>(CheckFailTerm)->setSuccessor(0, CheckTerm->getParent());
}

bool HWAddressSanitizer::ignoreMemIntrinsic(MemIntrinsic *MI) {
  if (MemTransferInst *MTI = dyn_cast<MemTransferInst>(MI)) {
    return (!ClInstrumentWrites || ignoreAccess(MTI, MTI->getDest())) &&
           (!ClInstrumentReads || ignoreAccess(MTI, MTI->getSource()));
  }
  if (isa<MemSetInst>(MI))
    return !ClInstrumentWrites || ignoreAccess(MI, MI->getDest());
  return false;
}

void HWAddressSanitizer::instrumentMemIntrinsic(MemIntrinsic *MI) {
  IRBuilder<> IRB(MI);
  if (isa<MemTransferInst>(MI)) {
    if (UseMatchAllCallback) {
      IRB.CreateCall(
          isa<MemMoveInst>(MI) ? HWAsanMemmove : HWAsanMemcpy,
          {IRB.CreatePointerCast(MI->getOperand(0), IRB.getInt8PtrTy()),
           IRB.CreatePointerCast(MI->getOperand(1), IRB.getInt8PtrTy()),
           IRB.CreateIntCast(MI->getOperand(2), IntptrTy, false),
           ConstantInt::get(Int8Ty, *MatchAllTag)});
    } else {
      IRB.CreateCall(
          isa<MemMoveInst>(MI) ? HWAsanMemmove : HWAsanMemcpy,
          {IRB.CreatePointerCast(MI->getOperand(0), IRB.getInt8PtrTy()),
           IRB.CreatePointerCast(MI->getOperand(1), IRB.getInt8PtrTy()),
           IRB.CreateIntCast(MI->getOperand(2), IntptrTy, false)});
    }
  } else if (isa<MemSetInst>(MI)) {
    if (UseMatchAllCallback) {
      IRB.CreateCall(
          HWAsanMemset,
          {IRB.CreatePointerCast(MI->getOperand(0), IRB.getInt8PtrTy()),
           IRB.CreateIntCast(MI->getOperand(1), IRB.getInt32Ty(), false),
           IRB.CreateIntCast(MI->getOperand(2), IntptrTy, false),
           ConstantInt::get(Int8Ty, *MatchAllTag)});
    } else {
      IRB.CreateCall(
          HWAsanMemset,
          {IRB.CreatePointerCast(MI->getOperand(0), IRB.getInt8PtrTy()),
           IRB.CreateIntCast(MI->getOperand(1), IRB.getInt32Ty(), false),
           IRB.CreateIntCast(MI->getOperand(2), IntptrTy, false)});
    }
  }
  MI->eraseFromParent();
}

bool HWAddressSanitizer::instrumentMemAccess(InterestingMemoryOperand &O) {
  Value *Addr = O.getPtr();

  LLVM_DEBUG(dbgs() << "Instrumenting: " << O.getInsn() << "\n");

  if (O.MaybeMask)
    return false; // FIXME

  IRBuilder<> IRB(O.getInsn());
  if (!O.TypeStoreSize.isScalable() && isPowerOf2_64(O.TypeStoreSize) &&
      (O.TypeStoreSize / 8 <= (1ULL << (kNumberOfAccessSizes - 1))) &&
      (!O.Alignment || *O.Alignment >= Mapping.getObjectAlignment() ||
       *O.Alignment >= O.TypeStoreSize / 8)) {
    size_t AccessSizeIndex = TypeSizeToSizeIndex(O.TypeStoreSize);
    if (InstrumentWithCalls) {
      if (UseMatchAllCallback) {
        IRB.CreateCall(HwasanMemoryAccessCallback[O.IsWrite][AccessSizeIndex],
                       {IRB.CreatePointerCast(Addr, IntptrTy),
                        ConstantInt::get(Int8Ty, *MatchAllTag)});
      } else {
        IRB.CreateCall(HwasanMemoryAccessCallback[O.IsWrite][AccessSizeIndex],
                       IRB.CreatePointerCast(Addr, IntptrTy));
      }
    } else if (OutlinedChecks) {
      instrumentMemAccessOutline(Addr, O.IsWrite, AccessSizeIndex, O.getInsn());
    } else {
      instrumentMemAccessInline(Addr, O.IsWrite, AccessSizeIndex, O.getInsn());
    }
  } else {
    if (UseMatchAllCallback) {
      IRB.CreateCall(
          HwasanMemoryAccessCallbackSized[O.IsWrite],
          {IRB.CreatePointerCast(Addr, IntptrTy),
           IRB.CreateUDiv(IRB.CreateTypeSize(IntptrTy, O.TypeStoreSize),
                          ConstantInt::get(IntptrTy, 8)),
           ConstantInt::get(Int8Ty, *MatchAllTag)});
    } else {
      IRB.CreateCall(
          HwasanMemoryAccessCallbackSized[O.IsWrite],
          {IRB.CreatePointerCast(Addr, IntptrTy),
           IRB.CreateUDiv(IRB.CreateTypeSize(IntptrTy, O.TypeStoreSize),
                          ConstantInt::get(IntptrTy, 8))});
    }
  }
  untagPointerOperand(O.getInsn(), Addr);

  return true;
}

void HWAddressSanitizer::tagAlloca(IRBuilder<> &IRB, AllocaInst *AI, Value *Tag,
                                   size_t Size) {
  size_t AlignedSize = alignTo(Size, Mapping.getObjectAlignment());
  if (!UseShortGranules)
    Size = AlignedSize;

  Tag = IRB.CreateTrunc(Tag, Int8Ty);
  if (InstrumentWithCalls) {
    IRB.CreateCall(HwasanTagMemoryFunc,
                   {IRB.CreatePointerCast(AI, Int8PtrTy), Tag,
                    ConstantInt::get(IntptrTy, AlignedSize)});
  } else {
    size_t ShadowSize = Size >> Mapping.Scale;
    Value *AddrLong = untagPointer(IRB, IRB.CreatePointerCast(AI, IntptrTy));
    Value *ShadowPtr = memToShadow(AddrLong, IRB);
    // If this memset is not inlined, it will be intercepted in the hwasan
    // runtime library. That's OK, because the interceptor skips the checks if
    // the address is in the shadow region.
    // FIXME: the interceptor is not as fast as real memset. Consider lowering
    // llvm.memset right here into either a sequence of stores, or a call to
    // hwasan_tag_memory.
    if (ShadowSize)
      IRB.CreateMemSet(ShadowPtr, Tag, ShadowSize, Align(1));
    if (Size != AlignedSize) {
      const uint8_t SizeRemainder = Size % Mapping.getObjectAlignment().value();
      IRB.CreateStore(ConstantInt::get(Int8Ty, SizeRemainder),
                      IRB.CreateConstGEP1_32(Int8Ty, ShadowPtr, ShadowSize));
      IRB.CreateStore(Tag, IRB.CreateConstGEP1_32(
                               Int8Ty, IRB.CreatePointerCast(AI, Int8PtrTy),
                               AlignedSize - 1));
    }
  }
}

unsigned HWAddressSanitizer::retagMask(unsigned AllocaNo) {
  if (TargetTriple.getArch() == Triple::x86_64)
    return AllocaNo & TagMaskByte;

  // A list of 8-bit numbers that have at most one run of non-zero bits.
  // x = x ^ (mask << 56) can be encoded as a single armv8 instruction for these
  // masks.
  // The list does not include the value 255, which is used for UAR.
  //
  // Because we are more likely to use earlier elements of this list than later
  // ones, it is sorted in increasing order of probability of collision with a
  // mask allocated (temporally) nearby. The program that generated this list
  // can be found at:
  // https://github.com/google/sanitizers/blob/master/hwaddress-sanitizer/sort_masks.py
  static const unsigned FastMasks[] = {
      0,   128, 64, 192, 32,  96,  224, 112, 240, 48, 16,  120,
      248, 56,  24, 8,   124, 252, 60,  28,  12,  4,  126, 254,
      62,  30,  14, 6,   2,   127, 63,  31,  15,  7,  3,   1};
  return FastMasks[AllocaNo % std::size(FastMasks)];
}

Value *HWAddressSanitizer::applyTagMask(IRBuilder<> &IRB, Value *OldTag) {
  if (TagMaskByte == 0xFF)
    return OldTag; // No need to clear the tag byte.
  return IRB.CreateAnd(OldTag,
                       ConstantInt::get(OldTag->getType(), TagMaskByte));
}

Value *HWAddressSanitizer::getNextTagWithCall(IRBuilder<> &IRB) {
  return IRB.CreateZExt(IRB.CreateCall(HwasanGenerateTagFunc), IntptrTy);
}

Value *HWAddressSanitizer::getStackBaseTag(IRBuilder<> &IRB) {
  if (ClGenerateTagsWithCalls)
    return nullptr;
  if (StackBaseTag)
    return StackBaseTag;
  // Extract some entropy from the stack pointer for the tags.
  // Take bits 20..28 (ASLR entropy) and xor with bits 0..8 (these differ
  // between functions).
  Value *StackPointerLong = getSP(IRB);
  Value *StackTag =
      applyTagMask(IRB, IRB.CreateXor(StackPointerLong,
                                      IRB.CreateLShr(StackPointerLong, 20)));
  StackTag->setName("hwasan.stack.base.tag");
  return StackTag;
}

Value *HWAddressSanitizer::getAllocaTag(IRBuilder<> &IRB, Value *StackTag,
                                        unsigned AllocaNo) {
  if (ClGenerateTagsWithCalls)
    return getNextTagWithCall(IRB);
  return IRB.CreateXor(
      StackTag, ConstantInt::get(StackTag->getType(), retagMask(AllocaNo)));
}

Value *HWAddressSanitizer::getUARTag(IRBuilder<> &IRB) {
  Value *StackPointerLong = getSP(IRB);
  Value *UARTag =
      applyTagMask(IRB, IRB.CreateLShr(StackPointerLong, PointerTagShift));

  UARTag->setName("hwasan.uar.tag");
  return UARTag;
}

// Add a tag to an address.
Value *HWAddressSanitizer::tagPointer(IRBuilder<> &IRB, Type *Ty,
                                      Value *PtrLong, Value *Tag) {
  assert(!UsePageAliases);
  Value *TaggedPtrLong;
  if (CompileKernel) {
    // Kernel addresses have 0xFF in the most significant byte.
    Value *ShiftedTag =
        IRB.CreateOr(IRB.CreateShl(Tag, PointerTagShift),
                     ConstantInt::get(IntptrTy, (1ULL << PointerTagShift) - 1));
    TaggedPtrLong = IRB.CreateAnd(PtrLong, ShiftedTag);
  } else {
    // Userspace can simply do OR (tag << PointerTagShift);
    Value *ShiftedTag = IRB.CreateShl(Tag, PointerTagShift);
    TaggedPtrLong = IRB.CreateOr(PtrLong, ShiftedTag);
  }
  return IRB.CreateIntToPtr(TaggedPtrLong, Ty);
}

// Remove tag from an address.
Value *HWAddressSanitizer::untagPointer(IRBuilder<> &IRB, Value *PtrLong) {
  assert(!UsePageAliases);
  Value *UntaggedPtrLong;
  if (CompileKernel) {
    // Kernel addresses have 0xFF in the most significant byte.
    UntaggedPtrLong =
        IRB.CreateOr(PtrLong, ConstantInt::get(PtrLong->getType(),
                                               TagMaskByte << PointerTagShift));
  } else {
    // Userspace addresses have 0x00.
    UntaggedPtrLong = IRB.CreateAnd(
        PtrLong, ConstantInt::get(PtrLong->getType(),
                                  ~(TagMaskByte << PointerTagShift)));
  }
  return UntaggedPtrLong;
}

Value *HWAddressSanitizer::getHwasanThreadSlotPtr(IRBuilder<> &IRB, Type *Ty) {
  Module *M = IRB.GetInsertBlock()->getParent()->getParent();
  if (TargetTriple.isAArch64() && TargetTriple.isAndroid()) {
    // Android provides a fixed TLS slot for sanitizers. See TLS_SLOT_SANITIZER
    // in Bionic's libc/private/bionic_tls.h.
    Function *ThreadPointerFunc =
        Intrinsic::getDeclaration(M, Intrinsic::thread_pointer);
    Value *SlotPtr = IRB.CreatePointerCast(
        IRB.CreateConstGEP1_32(Int8Ty, IRB.CreateCall(ThreadPointerFunc), 0x30),
        Ty->getPointerTo(0));
    return SlotPtr;
  }
  if (ThreadPtrGlobal)
    return ThreadPtrGlobal;

  return nullptr;
}

Value *HWAddressSanitizer::getPC(IRBuilder<> &IRB) {
  if (TargetTriple.getArch() == Triple::aarch64)
    return readRegister(IRB, "pc");
  return IRB.CreatePtrToInt(IRB.GetInsertBlock()->getParent(), IntptrTy);
}

Value *HWAddressSanitizer::getSP(IRBuilder<> &IRB) {
  if (!CachedSP) {
    // FIXME: use addressofreturnaddress (but implement it in aarch64 backend
    // first).
    Function *F = IRB.GetInsertBlock()->getParent();
    Module *M = F->getParent();
    auto *GetStackPointerFn = Intrinsic::getDeclaration(
        M, Intrinsic::frameaddress,
        IRB.getInt8PtrTy(M->getDataLayout().getAllocaAddrSpace()));
    CachedSP = IRB.CreatePtrToInt(
        IRB.CreateCall(GetStackPointerFn, {Constant::getNullValue(Int32Ty)}),
        IntptrTy);
  }
  return CachedSP;
}

Value *HWAddressSanitizer::getFrameRecordInfo(IRBuilder<> &IRB) {
  // Prepare ring buffer data.
  Value *PC = getPC(IRB);
  Value *SP = getSP(IRB);

  // Mix SP and PC.
  // Assumptions:
  // PC is 0x0000PPPPPPPPPPPP  (48 bits are meaningful, others are zero)
  // SP is 0xsssssssssssSSSS0  (4 lower bits are zero)
  // We only really need ~20 lower non-zero bits (SSSS), so we mix like this:
  //       0xSSSSPPPPPPPPPPPP
  SP = IRB.CreateShl(SP, 44);
  return IRB.CreateOr(PC, SP);
}

void HWAddressSanitizer::emitPrologue(IRBuilder<> &IRB, bool WithFrameRecord) {
  if (!Mapping.InTls)
    ShadowBase = getShadowNonTls(IRB);
  else if (!WithFrameRecord && TargetTriple.isAndroid())
    ShadowBase = getDynamicShadowIfunc(IRB);

  if (!WithFrameRecord && ShadowBase)
    return;

  Value *SlotPtr = nullptr;
  Value *ThreadLong = nullptr;
  Value *ThreadLongMaybeUntagged = nullptr;

  auto getThreadLongMaybeUntagged = [&]() {
    if (!SlotPtr)
      SlotPtr = getHwasanThreadSlotPtr(IRB, IntptrTy);
    if (!ThreadLong)
      ThreadLong = IRB.CreateLoad(IntptrTy, SlotPtr);
    // Extract the address field from ThreadLong. Unnecessary on AArch64 with
    // TBI.
    return TargetTriple.isAArch64() ? ThreadLong
                                    : untagPointer(IRB, ThreadLong);
  };

  if (WithFrameRecord) {
    switch (ClRecordStackHistory) {
    case libcall: {
      // Emit a runtime call into hwasan rather than emitting instructions for
      // recording stack history.
      Value *FrameRecordInfo = getFrameRecordInfo(IRB);
      IRB.CreateCall(HwasanRecordFrameRecordFunc, {FrameRecordInfo});
      break;
    }
    case instr: {
      ThreadLongMaybeUntagged = getThreadLongMaybeUntagged();

      StackBaseTag = IRB.CreateAShr(ThreadLong, 3);

      // Store data to ring buffer.
      Value *FrameRecordInfo = getFrameRecordInfo(IRB);
      Value *RecordPtr = IRB.CreateIntToPtr(ThreadLongMaybeUntagged,
                                            IntptrTy->getPointerTo(0));
      IRB.CreateStore(FrameRecordInfo, RecordPtr);

      // Update the ring buffer. Top byte of ThreadLong defines the size of the
      // buffer in pages, it must be a power of two, and the start of the buffer
      // must be aligned by twice that much. Therefore wrap around of the ring
      // buffer is simply Addr &= ~((ThreadLong >> 56) << 12).
      // The use of AShr instead of LShr is due to
      //   https://bugs.llvm.org/show_bug.cgi?id=39030
      // Runtime library makes sure not to use the highest bit.
      Value *WrapMask = IRB.CreateXor(
          IRB.CreateShl(IRB.CreateAShr(ThreadLong, 56), 12, "", true, true),
          ConstantInt::get(IntptrTy, (uint64_t)-1));
      Value *ThreadLongNew = IRB.CreateAnd(
          IRB.CreateAdd(ThreadLong, ConstantInt::get(IntptrTy, 8)), WrapMask);
      IRB.CreateStore(ThreadLongNew, SlotPtr);
      break;
    }
    case none: {
      llvm_unreachable(
          "A stack history recording mode should've been selected.");
    }
    }
  }

  if (!ShadowBase) {
    if (!ThreadLongMaybeUntagged)
      ThreadLongMaybeUntagged = getThreadLongMaybeUntagged();

    // Get shadow base address by aligning RecordPtr up.
    // Note: this is not correct if the pointer is already aligned.
    // Runtime library will make sure this never happens.
    ShadowBase = IRB.CreateAdd(
        IRB.CreateOr(
            ThreadLongMaybeUntagged,
            ConstantInt::get(IntptrTy, (1ULL << kShadowBaseAlignment) - 1)),
        ConstantInt::get(IntptrTy, 1), "hwasan.shadow");
    ShadowBase = IRB.CreateIntToPtr(ShadowBase, Int8PtrTy);
  }
}

Value *HWAddressSanitizer::readRegister(IRBuilder<> &IRB, StringRef Name) {
  Module *M = IRB.GetInsertBlock()->getParent()->getParent();
  Function *ReadRegister =
      Intrinsic::getDeclaration(M, Intrinsic::read_register, IntptrTy);
  MDNode *MD = MDNode::get(*C, {MDString::get(*C, Name)});
  Value *Args[] = {MetadataAsValue::get(*C, MD)};
  return IRB.CreateCall(ReadRegister, Args);
}

bool HWAddressSanitizer::instrumentLandingPads(
    SmallVectorImpl<Instruction *> &LandingPadVec) {
  for (auto *LP : LandingPadVec) {
    IRBuilder<> IRB(LP->getNextNode());
    IRB.CreateCall(
        HWAsanHandleVfork,
        {readRegister(IRB, (TargetTriple.getArch() == Triple::x86_64) ? "rsp"
                                                                      : "sp")});
  }
  return true;
}

static bool isLifetimeIntrinsic(Value *V) {
  auto *II = dyn_cast<IntrinsicInst>(V);
  return II && II->isLifetimeStartOrEnd();
}

bool HWAddressSanitizer::instrumentStack(memtag::StackInfo &SInfo,
                                         Value *StackTag, Value *UARTag,
                                         const DominatorTree &DT,
                                         const PostDominatorTree &PDT,
                                         const LoopInfo &LI) {
  // Ideally, we want to calculate tagged stack base pointer, and rewrite all
  // alloca addresses using that. Unfortunately, offsets are not known yet
  // (unless we use ASan-style mega-alloca). Instead we keep the base tag in a
  // temp, shift-OR it into each alloca address and xor with the retag mask.
  // This generates one extra instruction per alloca use.
  unsigned int I = 0;

  for (auto &KV : SInfo.AllocasToInstrument) {
    auto N = I++;
    auto *AI = KV.first;
    memtag::AllocaInfo &Info = KV.second;
    IRBuilder<> IRB(AI->getNextNode());

    // Replace uses of the alloca with tagged address.
    Value *Tag = getAllocaTag(IRB, StackTag, N);
    Value *AILong = IRB.CreatePointerCast(AI, IntptrTy);
    Value *AINoTagLong = untagPointer(IRB, AILong);
    Value *Replacement = tagPointer(IRB, AI->getType(), AINoTagLong, Tag);
    std::string Name =
        AI->hasName() ? AI->getName().str() : "alloca." + itostr(N);
    Replacement->setName(Name + ".hwasan");

    size_t Size = memtag::getAllocaSizeInBytes(*AI);
    size_t AlignedSize = alignTo(Size, Mapping.getObjectAlignment());

    Value *AICast = IRB.CreatePointerCast(AI, Int8PtrTy);

    auto HandleLifetime = [&](IntrinsicInst *II) {
      // Set the lifetime intrinsic to cover the whole alloca. This reduces the
      // set of assumptions we need to make about the lifetime. Without this we
      // would need to ensure that we can track the lifetime pointer to a
      // constant offset from the alloca, and would still need to change the
      // size to include the extra alignment we use for the untagging to make
      // the size consistent.
      //
      // The check for standard lifetime below makes sure that we have exactly
      // one set of start / end in any execution (i.e. the ends are not
      // reachable from each other), so this will not cause any problems.
      II->setArgOperand(0, ConstantInt::get(Int64Ty, AlignedSize));
      II->setArgOperand(1, AICast);
    };
    llvm::for_each(Info.LifetimeStart, HandleLifetime);
    llvm::for_each(Info.LifetimeEnd, HandleLifetime);

    AI->replaceUsesWithIf(Replacement, [AICast, AILong](const Use &U) {
      auto *User = U.getUser();
      return User != AILong && User != AICast && !isLifetimeIntrinsic(User);
    });

    for (auto *DDI : Info.DbgVariableIntrinsics) {
      // Prepend "tag_offset, N" to the dwarf expression.
      // Tag offset logically applies to the alloca pointer, and it makes sense
      // to put it at the beginning of the expression.
      SmallVector<uint64_t, 8> NewOps = {dwarf::DW_OP_LLVM_tag_offset,
                                         retagMask(N)};
      for (size_t LocNo = 0; LocNo < DDI->getNumVariableLocationOps(); ++LocNo)
        if (DDI->getVariableLocationOp(LocNo) == AI)
          DDI->setExpression(DIExpression::appendOpsToArg(DDI->getExpression(),
                                                          NewOps, LocNo));
    }

    auto TagEnd = [&](Instruction *Node) {
      IRB.SetInsertPoint(Node);
      // When untagging, use the `AlignedSize` because we need to set the tags
      // for the entire alloca to original. If we used `Size` here, we would
      // keep the last granule tagged, and store zero in the last byte of the
      // last granule, due to how short granules are implemented.
      tagAlloca(IRB, AI, UARTag, AlignedSize);
    };
    // Calls to functions that may return twice (e.g. setjmp) confuse the
    // postdominator analysis, and will leave us to keep memory tagged after
    // function return. Work around this by always untagging at every return
    // statement if return_twice functions are called.
    bool StandardLifetime =
        SInfo.UnrecognizedLifetimes.empty() &&
        memtag::isStandardLifetime(Info.LifetimeStart, Info.LifetimeEnd, &DT,
                                   &LI, ClMaxLifetimes) &&
        !SInfo.CallsReturnTwice;
    if (DetectUseAfterScope && StandardLifetime) {
      IntrinsicInst *Start = Info.LifetimeStart[0];
      IRB.SetInsertPoint(Start->getNextNode());
      tagAlloca(IRB, AI, Tag, Size);
      if (!memtag::forAllReachableExits(DT, PDT, LI, Start, Info.LifetimeEnd,
                                        SInfo.RetVec, TagEnd)) {
        for (auto *End : Info.LifetimeEnd)
          End->eraseFromParent();
      }
    } else {
      tagAlloca(IRB, AI, Tag, Size);
      for (auto *RI : SInfo.RetVec)
        TagEnd(RI);
      // We inserted tagging outside of the lifetimes, so we have to remove
      // them.
      for (auto &II : Info.LifetimeStart)
        II->eraseFromParent();
      for (auto &II : Info.LifetimeEnd)
        II->eraseFromParent();
    }
    memtag::alignAndPadAlloca(Info, Mapping.getObjectAlignment());
  }
  for (auto &I : SInfo.UnrecognizedLifetimes)
    I->eraseFromParent();
  return true;
}

void HWAddressSanitizer::sanitizeFunction(Function &F,
                                          FunctionAnalysisManager &FAM) {
  if (&F == HwasanCtorFunction)
    return;

  if (!F.hasFnAttribute(Attribute::SanitizeHWAddress))
    return;

  LLVM_DEBUG(dbgs() << "Function: " << F.getName() << "\n");

  SmallVector<InterestingMemoryOperand, 16> OperandsToInstrument;
  SmallVector<MemIntrinsic *, 16> IntrinToInstrument;
  SmallVector<Instruction *, 8> LandingPadVec;

  memtag::StackInfoBuilder SIB(SSI);
  for (auto &Inst : instructions(F)) {
    if (InstrumentStack) {
      SIB.visit(Inst);
    }

    if (InstrumentLandingPads && isa<LandingPadInst>(Inst))
      LandingPadVec.push_back(&Inst);

    getInterestingMemoryOperands(&Inst, OperandsToInstrument);

    if (MemIntrinsic *MI = dyn_cast<MemIntrinsic>(&Inst))
      if (!ignoreMemIntrinsic(MI))
        IntrinToInstrument.push_back(MI);
  }

  memtag::StackInfo &SInfo = SIB.get();

  initializeCallbacks(*F.getParent());

  if (!LandingPadVec.empty())
    instrumentLandingPads(LandingPadVec);

  if (SInfo.AllocasToInstrument.empty() && F.hasPersonalityFn() &&
      F.getPersonalityFn()->getName() == kHwasanPersonalityThunkName) {
    // __hwasan_personality_thunk is a no-op for functions without an
    // instrumented stack, so we can drop it.
    F.setPersonalityFn(nullptr);
  }

  if (SInfo.AllocasToInstrument.empty() && OperandsToInstrument.empty() &&
      IntrinToInstrument.empty())
    return;

  assert(!ShadowBase);

  Instruction *InsertPt = &*F.getEntryBlock().begin();
  IRBuilder<> EntryIRB(InsertPt);
  emitPrologue(EntryIRB,
               /*WithFrameRecord*/ ClRecordStackHistory != none &&
                   Mapping.WithFrameRecord &&
                   !SInfo.AllocasToInstrument.empty());

  if (!SInfo.AllocasToInstrument.empty()) {
    const DominatorTree &DT = FAM.getResult<DominatorTreeAnalysis>(F);
    const PostDominatorTree &PDT = FAM.getResult<PostDominatorTreeAnalysis>(F);
    const LoopInfo &LI = FAM.getResult<LoopAnalysis>(F);
    Value *StackTag = getStackBaseTag(EntryIRB);
    Value *UARTag = getUARTag(EntryIRB);
    instrumentStack(SInfo, StackTag, UARTag, DT, PDT, LI);
  }

  // If we split the entry block, move any allocas that were originally in the
  // entry block back into the entry block so that they aren't treated as
  // dynamic allocas.
  if (EntryIRB.GetInsertBlock() != &F.getEntryBlock()) {
    InsertPt = &*F.getEntryBlock().begin();
    for (Instruction &I :
         llvm::make_early_inc_range(*EntryIRB.GetInsertBlock())) {
      if (auto *AI = dyn_cast<AllocaInst>(&I))
        if (isa<ConstantInt>(AI->getArraySize()))
          I.moveBefore(InsertPt);
    }
  }

  for (auto &Operand : OperandsToInstrument)
    instrumentMemAccess(Operand);

  if (ClInstrumentMemIntrinsics && !IntrinToInstrument.empty()) {
    for (auto *Inst : IntrinToInstrument)
      instrumentMemIntrinsic(Inst);
  }

  ShadowBase = nullptr;
  StackBaseTag = nullptr;
  CachedSP = nullptr;
}

void HWAddressSanitizer::instrumentGlobal(GlobalVariable *GV, uint8_t Tag) {
  assert(!UsePageAliases);
  Constant *Initializer = GV->getInitializer();
  uint64_t SizeInBytes =
      M.getDataLayout().getTypeAllocSize(Initializer->getType());
  uint64_t NewSize = alignTo(SizeInBytes, Mapping.getObjectAlignment());
  if (SizeInBytes != NewSize) {
    // Pad the initializer out to the next multiple of 16 bytes and add the
    // required short granule tag.
    std::vector<uint8_t> Init(NewSize - SizeInBytes, 0);
    Init.back() = Tag;
    Constant *Padding = ConstantDataArray::get(*C, Init);
    Initializer = ConstantStruct::getAnon({Initializer, Padding});
  }

  auto *NewGV = new GlobalVariable(M, Initializer->getType(), GV->isConstant(),
                                   GlobalValue::ExternalLinkage, Initializer,
                                   GV->getName() + ".hwasan");
  NewGV->copyAttributesFrom(GV);
  NewGV->setLinkage(GlobalValue::PrivateLinkage);
  NewGV->copyMetadata(GV, 0);
  NewGV->setAlignment(
      std::max(GV->getAlign().valueOrOne(), Mapping.getObjectAlignment()));

  // It is invalid to ICF two globals that have different tags. In the case
  // where the size of the global is a multiple of the tag granularity the
  // contents of the globals may be the same but the tags (i.e. symbol values)
  // may be different, and the symbols are not considered during ICF. In the
  // case where the size is not a multiple of the granularity, the short granule
  // tags would discriminate two globals with different tags, but there would
  // otherwise be nothing stopping such a global from being incorrectly ICF'd
  // with an uninstrumented (i.e. tag 0) global that happened to have the short
  // granule tag in the last byte.
  NewGV->setUnnamedAddr(GlobalValue::UnnamedAddr::None);

  // Descriptor format (assuming little-endian):
  // bytes 0-3: relative address of global
  // bytes 4-6: size of global (16MB ought to be enough for anyone, but in case
  // it isn't, we create multiple descriptors)
  // byte 7: tag
  auto *DescriptorTy = StructType::get(Int32Ty, Int32Ty);
  const uint64_t MaxDescriptorSize = 0xfffff0;
  for (uint64_t DescriptorPos = 0; DescriptorPos < SizeInBytes;
       DescriptorPos += MaxDescriptorSize) {
    auto *Descriptor =
        new GlobalVariable(M, DescriptorTy, true, GlobalValue::PrivateLinkage,
                           nullptr, GV->getName() + ".hwasan.descriptor");
    auto *GVRelPtr = ConstantExpr::getTrunc(
        ConstantExpr::getAdd(
            ConstantExpr::getSub(
                ConstantExpr::getPtrToInt(NewGV, Int64Ty),
                ConstantExpr::getPtrToInt(Descriptor, Int64Ty)),
            ConstantInt::get(Int64Ty, DescriptorPos)),
        Int32Ty);
    uint32_t Size = std::min(SizeInBytes - DescriptorPos, MaxDescriptorSize);
    auto *SizeAndTag = ConstantInt::get(Int32Ty, Size | (uint32_t(Tag) << 24));
    Descriptor->setComdat(NewGV->getComdat());
    Descriptor->setInitializer(ConstantStruct::getAnon({GVRelPtr, SizeAndTag}));
    Descriptor->setSection("hwasan_globals");
    Descriptor->setMetadata(LLVMContext::MD_associated,
                            MDNode::get(*C, ValueAsMetadata::get(NewGV)));
    appendToCompilerUsed(M, Descriptor);
  }

  Constant *Aliasee = ConstantExpr::getIntToPtr(
      ConstantExpr::getAdd(
          ConstantExpr::getPtrToInt(NewGV, Int64Ty),
          ConstantInt::get(Int64Ty, uint64_t(Tag) << PointerTagShift)),
      GV->getType());
  auto *Alias = GlobalAlias::create(GV->getValueType(), GV->getAddressSpace(),
                                    GV->getLinkage(), "", Aliasee, &M);
  Alias->setVisibility(GV->getVisibility());
  Alias->takeName(GV);
  GV->replaceAllUsesWith(Alias);
  GV->eraseFromParent();
}

void HWAddressSanitizer::instrumentGlobals() {
  std::vector<GlobalVariable *> Globals;
  for (GlobalVariable &GV : M.globals()) {
    if (GV.hasSanitizerMetadata() && GV.getSanitizerMetadata().NoHWAddress)
      continue;

    if (GV.isDeclarationForLinker() || GV.getName().startswith("llvm.") ||
        GV.isThreadLocal())
      continue;

    // Common symbols can't have aliases point to them, so they can't be tagged.
    if (GV.hasCommonLinkage())
      continue;

    // Globals with custom sections may be used in __start_/__stop_ enumeration,
    // which would be broken both by adding tags and potentially by the extra
    // padding/alignment that we insert.
    if (GV.hasSection())
      continue;

    Globals.push_back(&GV);
  }

  MD5 Hasher;
  Hasher.update(M.getSourceFileName());
  MD5::MD5Result Hash;
  Hasher.final(Hash);
  uint8_t Tag = Hash[0];

  assert(TagMaskByte >= 16);

  for (GlobalVariable *GV : Globals) {
    // Don't allow globals to be tagged with something that looks like a
    // short-granule tag, otherwise we lose inter-granule overflow detection, as
    // the fast path shadow-vs-address check succeeds.
    if (Tag < 16 || Tag > TagMaskByte)
      Tag = 16;
    instrumentGlobal(GV, Tag++);
  }
}

void HWAddressSanitizer::instrumentPersonalityFunctions() {
  // We need to untag stack frames as we unwind past them. That is the job of
  // the personality function wrapper, which either wraps an existing
  // personality function or acts as a personality function on its own. Each
  // function that has a personality function or that can be unwound past has
  // its personality function changed to a thunk that calls the personality
  // function wrapper in the runtime.
  MapVector<Constant *, std::vector<Function *>> PersonalityFns;
  for (Function &F : M) {
    if (F.isDeclaration() || !F.hasFnAttribute(Attribute::SanitizeHWAddress))
      continue;

    if (F.hasPersonalityFn()) {
      PersonalityFns[F.getPersonalityFn()->stripPointerCasts()].push_back(&F);
    } else if (!F.hasFnAttribute(Attribute::NoUnwind)) {
      PersonalityFns[nullptr].push_back(&F);
    }
  }

  if (PersonalityFns.empty())
    return;

  FunctionCallee HwasanPersonalityWrapper = M.getOrInsertFunction(
      "__hwasan_personality_wrapper", Int32Ty, Int32Ty, Int32Ty, Int64Ty,
      Int8PtrTy, Int8PtrTy, Int8PtrTy, Int8PtrTy, Int8PtrTy);
  FunctionCallee UnwindGetGR = M.getOrInsertFunction("_Unwind_GetGR", VoidTy);
  FunctionCallee UnwindGetCFA = M.getOrInsertFunction("_Unwind_GetCFA", VoidTy);

  for (auto &P : PersonalityFns) {
    std::string ThunkName = kHwasanPersonalityThunkName;
    if (P.first)
      ThunkName += ("." + P.first->getName()).str();
    FunctionType *ThunkFnTy = FunctionType::get(
        Int32Ty, {Int32Ty, Int32Ty, Int64Ty, Int8PtrTy, Int8PtrTy}, false);
    bool IsLocal = P.first && (!isa<GlobalValue>(P.first) ||
                               cast<GlobalValue>(P.first)->hasLocalLinkage());
    auto *ThunkFn = Function::Create(ThunkFnTy,
                                     IsLocal ? GlobalValue::InternalLinkage
                                             : GlobalValue::LinkOnceODRLinkage,
                                     ThunkName, &M);
    if (!IsLocal) {
      ThunkFn->setVisibility(GlobalValue::HiddenVisibility);
      ThunkFn->setComdat(M.getOrInsertComdat(ThunkName));
    }

    auto *BB = BasicBlock::Create(*C, "entry", ThunkFn);
    IRBuilder<> IRB(BB);
    CallInst *WrapperCall = IRB.CreateCall(
        HwasanPersonalityWrapper,
        {ThunkFn->getArg(0), ThunkFn->getArg(1), ThunkFn->getArg(2),
         ThunkFn->getArg(3), ThunkFn->getArg(4),
         P.first ? IRB.CreateBitCast(P.first, Int8PtrTy)
                 : Constant::getNullValue(Int8PtrTy),
         IRB.CreateBitCast(UnwindGetGR.getCallee(), Int8PtrTy),
         IRB.CreateBitCast(UnwindGetCFA.getCallee(), Int8PtrTy)});
    WrapperCall->setTailCall();
    IRB.CreateRet(WrapperCall);

    for (Function *F : P.second)
      F->setPersonalityFn(ThunkFn);
  }
}

void HWAddressSanitizer::ShadowMapping::init(Triple &TargetTriple,
                                             bool InstrumentWithCalls) {
  Scale = kDefaultShadowScale;
  if (TargetTriple.isOSFuchsia()) {
    // Fuchsia is always PIE, which means that the beginning of the address
    // space is always available.
    InGlobal = false;
    InTls = false;
    Offset = 0;
    WithFrameRecord = true;
  } else if (ClMappingOffset.getNumOccurrences() > 0) {
    InGlobal = false;
    InTls = false;
    Offset = ClMappingOffset;
    WithFrameRecord = false;
  } else if (ClEnableKhwasan || InstrumentWithCalls) {
    InGlobal = false;
    InTls = false;
    Offset = 0;
    WithFrameRecord = false;
  } else if (ClWithIfunc) {
    InGlobal = true;
    InTls = false;
    Offset = kDynamicShadowSentinel;
    WithFrameRecord = false;
  } else if (ClWithTls) {
    InGlobal = false;
    InTls = true;
    Offset = kDynamicShadowSentinel;
    WithFrameRecord = true;
  } else {
    InGlobal = false;
    InTls = false;
    Offset = kDynamicShadowSentinel;
    WithFrameRecord = false;
  }
}
