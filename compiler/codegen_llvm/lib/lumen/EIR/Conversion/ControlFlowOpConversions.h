#ifndef LUMEN_EIR_CONVERSION_CONTROLFLOW_OP_CONVERSION
#define LUMEN_EIR_CONVERSION_CONTROLFLOW_OP_CONVERSION

#include "lumen/EIR/Conversion/ConversionSupport.h"

namespace lumen {
namespace eir {
class BranchOpConversion;
class CondBranchOpConversion;
class CallOpConversion;
class InvokeOpConversion;
class LandingPadOp;
class ReturnOpConversion;
class ThrowOpConversion;
class UnreachableOpConversion;
class YieldOpConversion;
class YieldCheckOpConversion;
class ReceiveStartOpConversion;
class ReceiveWaitOpConversion;
class ReceiveMessageOpConversion;
class ReceiveDoneOpConversion;

void populateControlFlowOpConversionPatterns(OwningRewritePatternList &patterns,
                                             MLIRContext *context,
                                             EirTypeConverter &converter,
                                             TargetInfo &targetInfo);
}  // namespace eir
}  // namespace lumen

#endif  // LUMEN_EIR_CONVERSION_CONTROLFLOW_OP_CONVERSION
