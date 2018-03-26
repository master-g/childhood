.macro PlayA220
  lda #$FF
  sta APU_PULSE1_MAIN
  lda #%11011011
  sta APU_PULSE1_SWEEP
  lda #$A5
  sta APU_PULSE1_TIMELO
  lda #$AB
  sta APU_PULSE1_LEN
  lda #%00000001
  sta APU_STATUS
.endm
