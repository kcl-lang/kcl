// Call the LLD linker on different targets.
#include "lld/Common/Driver.h"

extern "C" bool LldMachOMain(const char *argv[], size_t length)
{
	llvm::ArrayRef<const char *> args(argv, length);

	return lld::mach_o::link(args, false, llvm::outs(), llvm::errs());
}

extern "C" bool LldELFMain(const char *argv[], size_t length)
{
	llvm::ArrayRef<const char *> args(argv, length);

	return lld::elf::link(args, false, llvm::outs(), llvm::errs());
}

extern "C" bool LldMinGWMain(const char *argv[], size_t length)
{
	llvm::ArrayRef<const char *> args(argv, length);

	return lld::mingw::link(args, false, llvm::outs(), llvm::errs());
}

extern "C" bool LldWasmMain(const char *argv[], size_t length)
{
	llvm::ArrayRef<const char *> args(argv, length);

	return lld::wasm::link(args, false, llvm::outs(), llvm::errs());
}
