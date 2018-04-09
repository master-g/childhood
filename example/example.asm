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
.include "ram.inc"      ; program RAM defines

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


; At the same time that we strobe bit 0, we initialize the ring counter
; so we're hitting two birds with one stone here
ReadJoyStick:
  lda #$01
  ; While the strobe bit is set, buttons will be continuously reloaded.
  ; This means that reading from JOYSTICK1 will only return the state of the
  ; first button: button A.
  sta JOYSTICK1
  sta buttons
  lsr a        ; now A is 0
  ; By storing 0 into JOYSTICK1, the strobe bit is cleared and the reloading stops.
  ; This allows all 8 buttons (newly reloaded) to be read from JOYSTICK1.
  sta JOYSTICK1
ReadJoyStickLoop:
  lda JOYSTICK1
  lsr a	       ; bit0 -> Carry
  rol buttons  ; Carry -> bit0; bit 7 -> Carry
  bcc ReadJoyStickLoop

CheckButtonLeft:
  lda buttons
  and #PAD_LEFT
  beq CheckButtonRight

  ldx #$00
MoveLeftLoop:
  lda $0203, x
  sec
  sbc #$01
  sta $0203, x

  inx
  inx
  inx
  inx
  cpx #$10
  bne MoveLeftLoop

CheckButtonRight:
  lda buttons
  and #PAD_RIGHT
  beq CheckButtonSelect

  ldx #$00
MoveRightLoop:
  lda $0203, x
  clc
  adc #$01
  sta $0203, x

  inx
  inx
  inx
  inx
  cpx #$10
  bne MoveRightLoop

CheckButtonSelect:
  lda buttons
  and #PAD_SELECT
  beq CheckButtonEnd

  ldx #$00
ChangePaletteLoop:
  lda $0202, x
  clc
  adc #$01
  cmp #$04
  bne SelectPalette
  lda #$00
SelectPalette:
  sta $0202, x

  inx
  inx
  inx
  inx
  cpx #$10
  bne ChangePaletteLoop

CheckButtonEnd:


  ; This is the PPU clean up section
  ; so rendering the next frame starts properly
  lda #%10010000  ; enable NMI, sprites, background from Pattern Table 0 and 1
  sta PPU_CTRL
  lda #%00011110  ; enable sprites, background, no clipping on left side
  sta PPU_MASK
  lda #$00        ; tell PPU there is no background scrolling
  sta PPU_SCROLL
  sta PPU_SCROLL


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
  cpx #$10
  bne LoadSpritesLoop


LoadBackground:
  lda PPU_STATUS        ; read PPU status to reset the high/low latch
  lda #$20
  sta PPU_ADDR
  lda #$00
  sta PPU_ADDR
  ldx #$00
LoadBackgroundLoop:
  lda background, x
  sta PPU_DATA
  inx
  cpx #$80
  bne LoadBackgroundLoop


LoadAttribute:
  lda PPU_STATUS
  lda #$23
  sta PPU_ADDR
  lda #$C0
  sta PPU_ADDR
  ldx #$00
LoadAttributeLoop:
  lda attribute, x
  sta PPU_DATA
  inx
  cpx #$10
  bne LoadAttributeLoop


  lda #%10010000    ; enable NMI
                    ; sprites from Pattern Table 0
                    ; background from Pattern Table 1
  sta PPU_CTRL

  lda #%00011110   ; enable sprites, background, no clipping on left side
  sta PPU_MASK


  ; and then run your program's main loop.
MainLoop:
  jmp MainLoop

;==============================================================================;
; background palette
.org $E000
palette:
  ; background palette
  .db $22,$29,$1A,$0F
  .db $22,$36,$17,$0F
  .db $22,$30,$21,$0F
  .db $22,$27,$17,$0F
  ; sprite palette
  .db $22,$16,$27,$18   ; mario
  .db $22,$30,$27,$19   ; luigi
  .db $22,$37,$27,$16   ; fire
  .db $22,$1A,$30,$27   ; bowser

sprites:
  ;   vert tile attr horiz
  .db $80, $32, $00, $80  ; sprite 0
  .db $80, $33, $00, $88  ; sprite 1
  .db $88, $34, $00, $80  ; sprite 2
  .db $88, $35, $00, $88  ; sprite 3

background:
  .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ;;row 1
  .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ;;all sky

  .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ;;row 2
  .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ;;all sky

  .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ;;row 1
  .db $24,$24,$24,$24,$24,$24,$24,$24,$36,$37,$24,$24,$24,$24,$24,$24  ;;all sky

  .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ;;row 2
  .db $24,$24,$24,$24,$24,$24,$24,$35,$25,$25,$38,$24,$24,$24,$24,$24  ;;all sky

  .db $24,$24,$24,$24,$45,$45,$24,$24,$45,$45,$45,$45,$45,$45,$24,$24  ;;row 3
  .db $24,$24,$24,$24,$24,$24,$24,$39,$3A,$3B,$3C,$24,$53,$54,$24,$24  ;;some brick tops

  .db $24,$24,$24,$24,$47,$47,$24,$24,$47,$47,$47,$47,$47,$47,$24,$24  ;;row 4
  .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$55,$56,$24,$24  ;;brick bottoms

  .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ;;row 1
  .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ;;all sky

  .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ;;row 2
  .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ;;all sky

attribute:
  .db %00000000, %00000000, %00000000, %00000000, %00000000, %10000000, %10100000, %00000000
  .db %00000000, %00000010, %00000101, %00000001, %00000000, %00001000, %00001010, %00000011

  .db $24,$24,$24,$24, $47,$47,$24,$24 ,$47,$47,$47,$47, $47,$47,$24,$24
  .db $24,$24,$24,$24 ,$24,$24,$24,$24, $24,$24,$24,$24, $55,$56,$24,$24

;==============================================================================;
; Vectors
.org $FFFA
  .dw NMI
  .dw Reset
  .dw IRQ

;==============================================================================;
; CHR-ROM (if needed)
.incbin "mario.chr"
