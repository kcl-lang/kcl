# KCL excpetion
This readme mainly introduces the KCL exceptions and KCL exception message work flow.

## 1. Work Flow
```
                                             ----------------     ----------------      -----------------------      ------------------
                                             |              |     |              |      |                     |      |                |
                                             | kcl_error.py | --> | kcl_error.py | ---> | kcl_err_template.py | ---> | kcl_err_msg.py |
                                    no color |              |     |              |      |                     |      |                |
         KCLXXXError.gen_err_msg() <-------- |              |     |              |      |       no color      |      |                |
                                     color   | KCLXXXError  | <-- | KCLException | <--- |  <----------------  | <--- |                |
 KCLXXXError.show_msg_with_theme() <-------- |              |     |              |      |                     |      |                |
                                             ----------------     ----------------      -----------------------      ------------------
                                                                                            ^           | color
                                                                                            |           v
                                                                                       -----------------------      
                                                                                       |  kcl_err_theme.py   |
                                                                                       |                     |
                                                                                       -----------------------      
             
             
```
- KCLXXXError.gen_err_msg()
  1. `KCLXXXError` will call method `gen_err_msg()` of the parent class `KCLException`
  and get KCL error message without highlighting. 
  
  2. The `gen_err_msg()` of `KCLException` will call methods provided in `kcl_err_template.py` 
  to obtain the structured KCL error message. 
  
  3. `kcl_err_template.py` will obtain KCL error message constants through methods provided by `kcl_err_msg.py` 
  and return the structured error messages to `KCLException`. 
     
  4. Because `gen_err_msg()` sets the highlight flag to False, 
     so `kcl_err_template.py` will return KCL error messages without highlight.
     

- KCLXXXError.show_msg_with_theme()
  1. `KCLXXXError` will call method `show_msg_with_theme()` of the parent class `KCLException`
  and get KCL error message with highlighting. 
  
  2. The `gen_err_msg()` of `KCLException` will call methods provided in `kcl_err_template.py` 
  to obtain the structured KCL error message. 
  
  3. `kcl_err_template.py` will obtain KCL error message constants through methods provided by `kcl_err_msg.py` 
  and return the structured and highlight error messages to `KCLException`. 
     
  4. Because `show_msg_with_theme()` sets the highlight flag to True, 
     so before returning KCL error messages to `KCLException`, 
     `kcl_err_template.py` will call the method provided in `kcl_err_theme.py` to highlight KCL error messages.
     Then `kcl_err_template.py` will return KCL error messages with highlight.

## 2. KCL exceptions

This section lists all the exceptions in KCLVM. 
They can be divided into a three-level inheritance structure. 
The top-level KCLException inherits Python Exception, 
the second level contains 12 categories of KCL exceptions, 
and the last level contains 44 exceptions inherits the second level exceptions。

Each exception in KCL contains a unique identifier `ewcode`.
Each bit in ewcode shows different information in this exception.

For example: the ewcode of exception `FailedLoadModuleError` in kclvm is `E2F05`.
```
E: Error : This exception is an error.
2: Compile : This exception occured during compiling
F: Import : This exception occurred when importing the package
05: FailedLoadModule : This exception occurred because the module failed to load
```

All exceptions, the inheritance relationship between exceptions, 
and ewcode are shown in the following table.

| ewcode | KCL exception | parent exception |
| ---- | ---- | ---- |
| 00000 | KCLException | Exception |
| E0000 | KCLError | KCLException |
| W0000 | KCLWarning | KCLException |
| 01000 | KCLSyntaxException | KCLException |
| 02000 | KCLCompileException | KCLException |
| 03000 | KCLRuntimeException | KCLException |
| 00A00 | KCLAttributeException | KCLException |
| 00B00 | KCLSchemaException | KCLException |
| 00C00 | KCLMixinException | KCLException |
| 00D00 | KCLInheritException | KCLException |
| 00F00 | KCLImportException | KCLException |
| 00G00 | KCLTypeException | KCLException |
| 00H00 | KCLDecoratorException | KCLException |
| 00I00 | KCLArgumentException | KCLException |
| 00K00 | KCLOverflowException | KCLException |
| 00L00 | KCLComplingException | KCLException |
| 00M00 | KCLRunningException| KCLException |
| 00N00 | KCLDeprecatedException | KCLException |
| 00P00 | KCLDocException | KCLException |
| 00Q00 | KCLImmutableException | KCLException |
| E1001 | InvalidSyntaxError | KCLError, KCLSyntaxException |
| E1002 | KCLTabError | KCLError, KCLSyntaxException |
| E1003 | KCLIndentationError | KCLError, KCLSyntaxException |
| E2F04 | CannotFindModule | KCLError, KCLCompileException, KCLImportException |
| E2F05 | FailedLoadModule | KCLError, KCLCompileException, KCLImportException |
| E3F06 | RecursiveLoad | KCLError, KCLRuntimeException, KCLImportException |
| E3K04 | FloatOverflow | KCLError, KCLRuntimeException, KCLOverflowException |
| W2K04 | FloatUnderflow | KCLWarning, KCLCompileException, KCLOverflowException |
| E3K09 | IntOverflow | KCLError, KCLRuntimeException, KCLOverflowException |
| W2P10 | InvalidDocstring | KCLWarning, KCLCompileException, KCLDocException |
| E3N11 | DeprecatedError | KCLError, KCLRuntimeException, KCLDeprecatedException |
| W2N12 | DeprecatedWarning | KCLWarning, KCLDeprecatedException |
| E2H13 | UnKnownDecoratorError | KCLError, KCLCompileException, KCLDecoratorException |
| E2H14 | InvalidDecoratorTargetError | KCLError, KCLCompileException, KCLDecoratorException |
| E2C15 | MixinNamingError | KCLError, KCLCompileException, KCLMixinException |
| E2C16 | MixinStructureIllegal | KCLError, KCLCompileException, KCLMixinException |
| E3B17 | SchemaCheckFailure | KCLError, KCLRuntimeException, KCLSchemaException |
| E2B17 | CannotAddMembersComplieError | KCLError, KCLCompileException, KCLSchemaException |
| E3B19 | CannotAddMembersRuntimeError | KCLError, KCLRuntimeException, KCLSchemaException |
| E2B20 | IndexSignatureError | KCLError, KCLCompileException, KCLSchemaException |
| E3G21 | TypeRuntimeError | KCLError, KCLRuntimeException, KCLTypeException |
| E2G22 | TypeComplieError | KCLError, KCLCompileException, KCLTypeException |
| E2L23 | CompileError | KCLError, KCLCompileException, KCLComplingException |
| E2L24 | SelectorError | KCLError, KCLCompileException, KCLComplingException |
| E2L25 | KCLNameError | KCLError, KCLCompileException, KCLComplingException |
| E2L26 | KCLValueError | KCLError, KCLCompileException, KCLComplingException |
| E2L27 | KCLKeyError | KCLError, KCLCompileException, KCLComplingException |
| E2L28 | UniqueKeyError | KCLError, KCLCompileException, KCLComplingException |
| E2A29 | KCLAttributeComplieError | KCLError, KCLCompileException, KCLAttributeException |
| E3A30 | KCLAttributeRuntimeError | KCLError, KCLRuntimeException, KCLAttributeException |
| E2A31 | IllegalAttributeError | KCLError, KCLCompileException, KCLAttributeException |
| E2D32 | MultiInheritError | KCLError, KCLCompileException, KCLInheritException |
| E2D33 | CycleInheritError | KCLError, KCLRuntimeException, KCLInheritException |
| E2D34 | IllegalInheritError | KCLError, KCLCompileException, KCLInheritException |
| E3I35 | IllegalArgumentRuntimeError | KCLError, KCLRuntimeException, KCLArgumentException |
| E2I36 | IllegalArgumentComplieError | KCLError, KCLCompileException, KCLArgumentException |
| E1I37 | IllegalArgumentSyntaxError | KCLError, KCLSyntaxException, KCLArgumentException |
| E3M38 | EvaluationError | KCLError, KCLRuntimeException, KCLRunningException |
| E3M39 | InvalidFormatSpec | KCLError, KCLRuntimeException, KCLRunningException |
| E3M40 | KCLAssertionError | KCLError, KCLRuntimeException, KCLRunningException |
| E3M41 | ImmutableCompileError | KCLError, KCLCompileException, KCLImmutableException |
| E3M44 | ImmutableRuntimeError | KCLError, KCLRuntimeException, KCLImmutableException |
| E3M42 | KCLRecursionError | KCLError, KCLRuntimeException, KCLRunningException |
| E3M43 | PlanError | KCLError, KCLRuntimeException, KCLRunningException |
