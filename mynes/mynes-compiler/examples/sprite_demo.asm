;
; Sprite Demo Example for NES
; Demonstrates sprite movement and animation on the NES
;

.inesprg 1      ; 1x 16KB PRG ROM bank
.ineschr 1      ; 1x 8KB CHR ROM bank
.inesmap 0      ; NROM mapper
.inesmir 1      ; Vertical mirroring

; Constants
SPRITE_COUNT = 4
SPRITE_SIZE = 4     ; 4 bytes per sprite (Y, tile, attributes, X)

; Controller bits
BUTTON_A      = %10000000
BUTTON_B      = %01000000
BUTTON_SELECT = %00100000
BUTTON_START  = %00010000
BUTTON_UP     = %00001000
BUTTON_DOWN   = %00000100
BUTTON_LEFT   = %00000010
BUTTON_RIGHT  = %00000001

; Variables in zero page
.zp
sprite_x:       .rs 1   ; Player sprite X position
sprite_y:       .rs 1   ; Player sprite Y position
sprite_dir:     .rs 1   ; Sprite direction (0-3)
animation_frame: .rs 1  ; Current animation frame
animation_counter: .rs 1 ; Animation timing counter
controller1:    .rs 1   ; Controller 1 state
controller1_old: .rs 1  ; Previous controller state
temp:           .rs 1   ; Temporary variable

; Variables in RAM
.bss
.org $0200
sprite_ram:     .rs 256 ; OAM (Object Attribute Memory) buffer

; Main program code
.code
.org $8000

; Reset vector entry point
RESET:
    ; Standard NES initialization
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
clear_memory:
    sta $0000, x
    sta $0100, x
    sta $0300, x
    sta $0400, x
    sta $0500, x
    sta $0600, x
    sta $0700, x
    lda #$FE
    sta sprite_ram, x  ; Clear sprite memory (off-screen)
    lda #0
    inx
    bne clear_memory

    ; Second vblank wait
vblankwait2:
    bit PPUSTATUS
    bpl vblankwait2

    ; Initialize variables
    lda #120           ; Center of screen
    sta sprite_x
    lda #112
    sta sprite_y
    lda #0
    sta sprite_dir
    sta animation_frame
    sta animation_counter

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

    ; Clear background
    lda PPUSTATUS      ; Reset PPU address latch
    lda #$20
    sta PPUADDR        ; Set PPU address to nametable
    lda #$00
    sta PPUADDR

    lda #$00           ; Use tile 0 (blank)
    ldx #$00
    ldy #$00
clear_background:
    sta PPUDATA
    inx
    bne clear_background
    iny
    cpy #$04          ; 4 pages = 1024 bytes
    bne clear_background

    ; Initialize sprites
    jsr init_sprites

    ; Enable rendering
    lda #%10010000     ; Enable NMI, sprites use pattern table 0
    sta PPUCTRL
    lda #%00011110     ; Enable background and sprites
    sta PPUMASK

    ; Main game loop
main_loop:
    jsr read_controller
    jsr update_sprite
    jsr animate_sprite

    ; Wait for next frame
    lda #1
    sta temp
wait_nmi:
    lda temp
    bne wait_nmi

    jmp main_loop

; Read controller input
read_controller:
    ; Save previous state
    lda controller1
    sta controller1_old

    ; Strobe controller
    lda #1
    sta $4016
    lda #0
    sta $4016

    ; Read 8 buttons
    ldx #8
read_buttons:
    lda $4016
    lsr a              ; Shift bit 0 into carry
    rol controller1    ; Rotate carry into controller1
    dex
    bne read_buttons
    rts

; Update sprite position based on controller input
update_sprite:
    ; Check for movement
    lda controller1
    and #BUTTON_UP
    beq check_down
    ; Move up
    lda sprite_y
    cmp #8             ; Top boundary
    bcc no_move_up
    dec sprite_y
    lda #0             ; Set direction to up
    sta sprite_dir
no_move_up:

check_down:
    lda controller1
    and #BUTTON_DOWN
    beq check_left
    ; Move down
    lda sprite_y
    cmp #224           ; Bottom boundary
    bcs no_move_down
    inc sprite_y
    lda #2             ; Set direction to down
    sta sprite_dir
no_move_down:

check_left:
    lda controller1
    and #BUTTON_LEFT
    beq check_right
    ; Move left
    lda sprite_x
    cmp #8             ; Left boundary
    bcc no_move_left
    dec sprite_x
    lda #3             ; Set direction to left
    sta sprite_dir
no_move_left:

check_right:
    lda controller1
    and #BUTTON_RIGHT
    beq done_movement
    ; Move right
    lda sprite_x
    cmp #248           ; Right boundary
    bcs no_move_right
    inc sprite_x
    lda #1             ; Set direction to right
    sta sprite_dir
no_move_right:

done_movement:
    rts

; Animate sprite based on movement and time
animate_sprite:
    ; Increment animation counter
    inc animation_counter
    lda animation_counter
    cmp #8             ; Change frame every 8 game frames
    bcc no_frame_change

    ; Reset counter and advance frame
    lda #0
    sta animation_counter
    inc animation_frame
    lda animation_frame
    cmp #4             ; 4 animation frames
    bcc no_frame_wrap
    lda #0
    sta animation_frame
no_frame_wrap:

no_frame_change:
    ; Update sprite in OAM
    lda sprite_y
    sta sprite_ram     ; Y position

    ; Calculate tile based on direction and frame
    lda sprite_dir
    asl a              ; Multiply by 4
    asl a
    clc
    adc animation_frame ; Add frame offset
    adc #$10           ; Base sprite tile
    sta sprite_ram + 1 ; Tile number

    lda #%00000000     ; Attributes (palette 0, no flip)
    sta sprite_ram + 2

    lda sprite_x
    sta sprite_ram + 3 ; X position

    rts

; Initialize sprites (hide unused sprites)
init_sprites:
    ldx #4             ; Start after first sprite
    lda #$FE           ; Off-screen Y position
init_loop:
    sta sprite_ram, x
    inx
    inx
    inx
    inx
    bne init_loop
    rts

; NMI handler (called during vertical blank)
NMI:
    ; Save registers
    pha
    txa
    pha
    tya
    pha

    ; Copy sprite data to PPU OAM
    lda #$00
    sta $2003          ; Set OAM address to 0
    lda #>sprite_ram   ; High byte of sprite_ram
    sta $4014          ; Start DMA transfer

    ; Clear the waiting flag
    lda #0
    sta temp

    ; Restore registers and return
    pla
    tay
    pla
    tax
    pla
    rti

; IRQ handler (not used)
IRQ:
    rti

; Data section
.data

; Color palette
palette:
    ; Background palette
    .db $0F,$00,$10,$30  ; Black, dark gray, light gray, white
    .db $0F,$01,$11,$21  ; Black, blue shades
    .db $0F,$06,$16,$26  ; Black, red shades
    .db $0F,$09,$19,$29  ; Black, green shades

    ; Sprite palette
    .db $0F,$07,$17,$27  ; Black, brown shades (character)
    .db $0F,$02,$12,$22  ; Black, blue shades
    .db $0F,$04,$14,$24  ; Black, purple shades
    .db $0F,$08,$18,$28  ; Black, yellow shades

; Vectors
.org $FFFA
    .dw NMI    ; NMI vector
    .dw RESET  ; Reset vector
    .dw IRQ    ; IRQ vector

; Character ROM data
.bank 1
.org $0000

; Tile 0: Blank
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$00,$00,$00,$00,$00,$00,$00

; Tiles 1-15: Simple background tiles
.org $0010

; Simple character sprites (tiles $10-$2F)
; Sprite facing up (tiles $10-$13)
.db $00,$3C,$7E,$FF,$E7,$E7,$7E,$3C  ; Frame 0
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$3C,$7E,$FF,$E7,$E7,$7E,$3C  ; Frame 1
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$3C,$7E,$FF,$E7,$E7,$7E,$3C  ; Frame 2
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$3C,$7E,$FF,$E7,$E7,$7E,$3C  ; Frame 3
.db $00,$00,$00,$00,$00,$00,$00,$00

; Sprite facing right (tiles $14-$17)
.db $00,$1E,$3F,$7F,$73,$73,$3F,$1E  ; Frame 0
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$1E,$3F,$7F,$73,$73,$3F,$1E  ; Frame 1
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$1E,$3F,$7F,$73,$73,$3F,$1E  ; Frame 2
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$1E,$3F,$7F,$73,$73,$3F,$1E  ; Frame 3
.db $00,$00,$00,$00,$00,$00,$00,$00

; Sprite facing down (tiles $18-$1B)
.db $00,$3C,$7E,$FF,$E7,$E7,$7E,$3C  ; Frame 0
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$3C,$7E,$FF,$E7,$E7,$7E,$3C  ; Frame 1
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$3C,$7E,$FF,$E7,$E7,$7E,$3C  ; Frame 2
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$3C,$7E,$FF,$E7,$E7,$7E,$3C  ; Frame 3
.db $00,$00,$00,$00,$00,$00,$00,$00

; Sprite facing left (tiles $1C-$1F)
.db $00,$78,$FC,$FE,$CE,$CE,$FC,$78  ; Frame 0
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$78,$FC,$FE,$CE,$CE,$FC,$78  ; Frame 1
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$78,$FC,$FE,$CE,$CE,$FC,$78  ; Frame 2
.db $00,$00,$00,$00,$00,$00,$00,$00
.db $00,$78,$FC,$FE,$CE,$CE,$FC,$78  ; Frame 3
.db $00,$00,$00,$00,$00,$00,$00,$00

; Fill remaining CHR ROM
.org $1000
.ds $1000, $00
