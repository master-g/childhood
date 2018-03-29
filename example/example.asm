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

.include "sound.asm"

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
  lda #$00
  sta OAM_ADDR  ; set the low byte (00) of the RAM address
  lda #$02
  sta OAM_DMA   ; set the high byte (02) of the RAM address, start the transfer

LatchController:
  lda #$01
  sta JOYSTICK1
  lda #$00
  sta JOYSTICK1

ReadA:
  lda JOYSTICK1
  and #%00000001  ; only look at bit 0
  beq ReadADone

  lda $0203       ; load sprite X position
  clc             ; clear carry flag before add
  adc #$01        ; A = A + 1
  sta $0203       ; save sprite X position
ReadADone:    ; handling this button is done

ReadB:
  lda JOYSTICK1
  and #%00000001
  beq ReadBDone

  lda $0203
  sec             ; set carry flag before subtract
  sbc #$01        ; A = A - 1
  sta $0203
ReadBDone:

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

  bit PPU_STATUS
  ; wait for 1st vblank
@waitVBLANK1:
  bit PPU_STATUS
  bpl @waitVBLANK1

@clearRAM
  lda #$00
  sta $0000, x
  sta $0100, x
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
LoadPalettes:
  lda PPU_STATUS        ; read PPU status to reset the high/low latch
  lda #$3F
  sta PPU_ADDR          ; write the high byte of $3F00 address
  lda #$00
  sta PPU_ADDR          ; write the low byte of $3F00 address
                        ; PPU_DATA is now ready to accept data
  ldx #$00              ; start out at 0
LoadPaletteLoop:
  lda palette, x        ; load data from address (palette + the value in x)
                        ; 1st time through loop it will load palette+0
                        ; 2nd time through loop it will load palette+1
                        ; 3rd time through loop it will load palette+2
                        ; etc
  sta PPU_DATA          ; write to PPU
  inx                   ; X = X + 1
  cpx #$20              ; Compare X to hex $10, decimal 16
  bne LoadPaletteLoop

LoadSprites:
  ldx #$00              ; start at 0
LoadSpritesLoop:
  lda sprites, x        ; load data from address (sprites + x)
  sta $0200, x          ; store into RAM address ($200 + x)
  inx
  cpx #$20
  bne LoadSpritesLoop

  lda #%10000000   ; enable NMI, sprites from Pattern Table 0
  sta PPU_CTRL

  lda #%00010000   ; enable sprites
  sta PPU_MASK

  ; and then run your program's main loop.
MainLoop:
  jmp MainLoop

;==============================================================================;
; background palette
.org $E000
palette:
  .db $0F,$31,$32,$33,$0F,$35,$36,$37,$0F,$39,$3A,$3B,$0F,$3D,$3E,$0F
  .db $0F,$1C,$15,$14,$0F,$02,$38,$3C,$0F,$1C,$15,$14,$0F,$02,$38,$3C

sprites:
    ;vert tile attr horiz
  .db $80, $32, $00, $80   ;sprite 0
  .db $80, $33, $00, $88   ;sprite 1
  .db $88, $34, $00, $80   ;sprite 2
  .db $88, $35, $00, $88   ;sprite 3

;==============================================================================;
; Vectors
.org $FFFA
  .dw NMI
  .dw Reset
  .dw IRQ

;==============================================================================;
; CHR-ROM (if needed)
.incbin "mario.chr"
