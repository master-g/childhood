;
; Hello World Example for NES
; A simple "Hello, World!" program demonstrating basic NES programming
;

.inesprg 1      ; 1x 16KB PRG ROM bank
.ineschr 1      ; 1x 8KB CHR ROM bank
.inesmap 0      ; NROM mapper
.inesmir 1      ; Vertical mirroring

; iNES header constants
PRG_BANKS = 1
CHR_BANKS = 1
MIRRORING = 1
MAPPER = 0

; NES Hardware Registers (predefined in compiler)
; PPUCTRL   = $2000
; PPUMASK   = $2001
; PPUSTATUS = $2002
; PPUADDR   = $2006
; PPUDATA   = $2007

; Variables in zero page
.zp
cursor_x:       .rs 1   ; Cursor X position
cursor_y:       .rs 1   ; Cursor Y position
temp:           .rs 1   ; Temporary variable

; Main program code
.code
.org $8000

; Reset vector entry point
RESET:
    ; Disable interrupts and set up stack
    sei                 ; Disable interrupts
    cld                 ; Clear decimal mode
    ldx #$40
    stx $4017          ; Disable APU frame IRQ
    ldx #$FF
    txs                ; Set up stack pointer
    inx                ; X = 0
    stx PPUCTRL        ; Disable NMI
    stx PPUMASK        ; Disable rendering
    stx $4010          ; Disable DMC IRQs

    ; Wait for PPU to stabilize
    bit PPUSTATUS
vblankwait1:
    bit PPUSTATUS
    bpl vblankwait1

    ; Clear memory
    lda #0
    sta $0000, x
    sta $0100, x
    sta $0300, x
    sta $0400, x
    sta $0500, x
    sta $0600, x
    sta $0700, x
    lda #$FE
    sta $0200, x       ; Clear sprite memory
    inx
    bne vblankwait1    ; Use the clear loop for timing

    ; Second vblank wait
vblankwait2:
    bit PPUSTATUS
    bpl vblankwait2

    ; Load palette
    lda PPUSTATUS      ; Reset PPU address latch
    lda #$3F
    sta PPUADDR        ; Set PPU address to palette
    lda #$00
    sta PPUADDR

    ldx #0
load_palettes:
    lda palette, x
    sta PPUDATA
    inx
    cpx #32
    bne load_palettes

    ; Load background
    lda PPUSTATUS      ; Reset PPU address latch
    lda #$20
    sta PPUADDR        ; Set PPU address to nametable
    lda #$00
    sta PPUADDR

    ; Clear the screen first
    lda #$00           ; Use tile 0 (blank)
    ldx #$00
    ldy #$00
clear_screen:
    sta PPUDATA
    inx
    bne clear_screen
    iny
    cpy #$04          ; 4 pages = 1024 bytes
    bne clear_screen

    ; Write "HELLO, WORLD!" message
    lda PPUSTATUS      ; Reset PPU address latch
    lda #$21           ; Start at row 10, column 10
    sta PPUADDR        ; ($2000 + 10*32 + 10 = $214A)
    lda #$4A
    sta PPUADDR

    ldx #0
write_message:
    lda message, x
    beq done_message   ; If we hit 0, we're done
    sta PPUDATA
    inx
    jmp write_message
done_message:

    ; Enable rendering
    lda #%10000000     ; Enable NMI
    sta PPUCTRL
    lda #%00001110     ; Enable background and sprites
    sta PPUMASK

    ; Main game loop
main_loop:
    jmp main_loop      ; Infinite loop

; NMI handler (called during vertical blank)
NMI:
    ; Save registers
    pha
    txa
    pha
    tya
    pha

    ; NMI code would go here
    ; For this simple example, we don't need to do anything

    ; Restore registers and return
    pla
    tay
    pla
    tax
    pla
    rti

; IRQ handler (not used in this example)
IRQ:
    rti

; Data section
.data

; Color palette
palette:
    .db $22,$29,$1A,$0F,  $22,$36,$17,$0F,  $22,$30,$21,$0F,  $22,$27,$17,$0F  ; Background palette
    .db $22,$16,$27,$18,  $22,$1A,$30,$27,  $22,$16,$30,$27,  $22,$0F,$36,$17  ; Sprite palette

; Hello World message (using tile numbers)
; For simplicity, using ASCII values - 64 to get tile numbers
; Assumes font tiles are arranged starting from tile 1
message:
    .db "HELLO, WORLD!", 0

; Vectors (placed at end of ROM)
.org $FFFA
    .dw NMI    ; NMI vector
    .dw RESET  ; Reset vector
    .dw IRQ    ; IRQ vector

; Character ROM data
.bank 1
.org $0000

; Simple font data - just a few characters for the demo
; Each character is 8x8 pixels, 2 bits per pixel
; This is a very basic font - in a real game you'd have a complete character set

; Tile 0: Blank
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$00,$00,$00,$00,$00,$00,$00

; Simple letters for "HELLO, WORLD!"
; Note: This is greatly simplified - a real font would be much more detailed
; and would require proper tile layout

; For this example, we'll just use pattern data that creates recognizable letters
; Tile data would normally be generated from graphics tools

; Fill the rest of CHR ROM with blank tiles
.org $1000
.ds $1000, $00  ; Fill remaining CHR ROM with zeros
