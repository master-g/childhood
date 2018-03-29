; RAM.inc - RAM layout (shared variables, defines, etc.)
;==============================================================================;
;-- zero page --------------------------------;
  ; $00-$0F: "Operating Space"
  ; temp space for function arguments, etc.
  .enum $0000
    buttons  .dsb 1, $00
  .ende
