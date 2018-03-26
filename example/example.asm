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

DrawSprite:
  lda #$08      ; top of the screen
  sta $0200     ; sprite 1 y position
  lda #$08
  sta $0204     ; sprite 2 y position
  lda #$10
  sta $0208
  lda #$10
  sta $020C
  lda #$3A
  sta $0201
  lda #$37
  sta $0205
  lda #$4F
  sta $0209
  lda #$4F
  sta $020D
  lda #$00
  sta $0202
  sta $0206
  sta $020A
  lda #$40
  sta $020E
  lda #$08
  sta $0203
  lda #$10
  sta $0207
  lda #$08
  sta $020B
  lda #$10
  sta $020F

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
  ldx #$00              ; start out at 0
LoadBackgroundPaletteLoop:
  lda background_palette, x ; load data from address (palette + the value in x)
                            ; 1st time through loop it will load palette+0
                            ; 2nd time through loop it will load palette+1
                            ; 3rd time through loop it will load palette+2
                            ; etc
  sta PPU_DATA          ; write to PPU
  inx                   ; X = X + 1
  cpx #$10              ; Compare X to hex $10, decimal 16 - copying 16 bytes = 4 sprites
  bne LoadBackgroundPaletteLoop  ; Branch to LoadPalettesLoop if compare was Not Equal to ze

  ldx #$00
LoadSpritePaletteLoop:
  lda sprite_palette, x
  sta PPU_DATA
  inx
  cpx #$10
  bne LoadSpritePaletteLoop

  lda #%10000000  ; enable NMI, sprites from Palette Table 0
  sta PPU_CTRL

  lda #%00010000  ; enable sprite
  sta PPU_MASK

  ; and then run your program's main loop.
MainLoop:
  jmp MainLoop

;==============================================================================;
; background palette
.org $E000
background_palette:
  .db $22,$29,$1A,$0F ;background palette 1
  .db $22,$36,$17,$0F ;background palette 2
  .db $22,$30,$21,$0F ;background palette 3
  .db $22,$27,$17,$0F ;background palette 4

sprite_palette:
  .db $22,$16,$27,$18 ;sprite palette 1
  .db $22,$1A,$30,$27 ;sprite palette 2
  .db $22,$16,$30,$27 ;sprite palette 3
  .db $22,$0F,$36,$17 ;sprite palette 4

;==============================================================================;
; Vectors
.org $FFFA
  .dw NMI
  .dw Reset
  .dw IRQ

;==============================================================================;
; CHR-ROM (if needed)
.incbin "mario.chr"
