; freemco NES Corelib Example 00: Skeleton Project (NROM)
;==============================================================================;
; The skeleton project targets NROM, as the examples are meant to be small.
; Whether or not that's NROM-128 or NROM-256 is up to the example.
; Other examples may need to use a different mapper.

; This skeleton project makes a few assumptions that you might have to change
; when developing for other mappers.
;==============================================================================;
; iNES header (NROM-128)
.include "NROM-128.asm"

; defines
.include "nes.inc"      ; NES hardware defines

;==============================================================================;
; program code
; .org $8000          ; starting point for NROM-256
.org $C000          ; starting point for NROM-128

;==============================================================================;
; NMI

NMI:
  ; save registers
  pha            ; 1) push A
  txa
  pha            ; 2) push X
  tya
  pha            ; 3) push Y

  ; "proper" NMI code belongs here.

NMI_end:
  ; restore registers
  pla            ; 3) pull Y
  tay
  pla            ; 2) pull X
  tax
  pla            ; 1) pull A
  rti

;==============================================================================;
; IRQ
; The IRQ is rarely used in simple mapper situations (such as NROM), but IRQs
; can be toggled via the NES's APU. The skeleton example does not use it.

IRQ:
  rti

;==============================================================================;
; Reset
; Handles NES initialization

Reset:
  sei                   ; disable IRQs
  cld                   ; clear decimal mode
  ldx #$40
  stx APU_FRAMECOUNT    ; disable APU frame IRQ
  ldx #$FF
  txs                   ; set up stack at $01FF
  inx                   ; (X is now $00)
  stx PPU_CTRL          ; disable NMIs
  stx PPU_MASK          ; disable rendering
  stx APU_DMC_FREQ      ; disable DMC IRQs

  ; if you're using a mapper, you should probably initialize it here.

  ; wait for 1st vblank
@waitVBLANK1:
  bit PPU_STATUS
  bpl @waitVBLANK1

@clearRAM
  lda #$00
  sta $0000, x
  sta $0100, x
  sta $0200, x
  sta $0400, x
  sta $0500, x
  sta $0600, x
  sta $0700, x
  lda #$FE
  sta $0300, x
  inx
  bne @clearRAM

  ; at this point, you can start setting up your program.

  ; after setting up your program, wait for the 2nd vblank
@waitVBLANK2:
  bit PPU_STATUS
  bpl @waitVBLANK2

  ; perform final commands (setting up PPU)
  lda #%10000000 ; intensify blues
  sta PPU_MASK

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

  ; and then run your program's main loop.
MainLoop:
  PlayA220
  jmp MainLoop

;==============================================================================;
; Vectors
.org $FFFA
  .dw NMI
  .dw Reset
  .dw IRQ

;==============================================================================;
; CHR-ROM (if needed)
.incbin "mario.chr"
