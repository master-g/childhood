package main

import (
	"encoding/hex"
	"fmt"
	"image"
	"image/color"
	"image/png"
	"io"
	"math/bits"
	"os"

	flag "github.com/spf13/pflag"
)

const (
	kPaletteSize     = 64       // NES palette have 64 colors
	kRGBSize         = 3        // RGB 3 bytes
	kCHRSize         = 1024 * 8 // process 8KB per time
	kPageSizeInBytes = 256 * 16 // 16x16 tiles, 16 bytes per tile
)

var (
	palette       []byte
	spritePalette []byte
	outFile       string
)

func main() {
	chr := flag.String("chr", "", "chr file to convert")
	pal := flag.String("pal", "RGB", "palette format")
	sprpal := flag.String("sp", "22271618", "sprite palette")
	out := flag.String("out", "chr", "output file")
	flag.Parse()

	if *chr == "" || *out == "" {
		flag.Usage()
		os.Exit(86)
	}

	// load sprite palette
	loadSpritePalette(*sprpal)
	// load palette
	loadPalette(*pal)
	// process CHR file
	processCHR(*chr)
}

func loadSpritePalette(sp string) {
	var err error
	spritePalette, err = hex.DecodeString(sp)
	if err != nil {
		fmt.Println(err)
		os.Exit(-1)
	}
}

func loadPalette(paletteName string) {
	palette = getPalette(paletteName)
	if palette == nil {
		f, err := os.Open(paletteName)
		defer f.Close()
		if err != nil {
			fmt.Printf("'%v' is not a valid palette name or file\n", paletteName)
			fmt.Println("use one of the palettes below or a valid PAL file")
			for k, _ := range paletteMap {
				fmt.Println("    " + k)
			}
			os.Exit(-1)
		}
		palette = make([]byte, kPaletteSize*kRGBSize)
		n, err := f.Read(palette)
		if err != nil {
			fmt.Printf("error while reading palette file '%v'\n", err)
			os.Exit(-1)
		}
		if n < len(palette) {
			fmt.Printf("invalid PAL file, expect 192 bytes, got %v\n", n)
			os.Exit(-1)
		}
	}
}

func processCHR(fileName string) {
	inFile, err := os.Open(fileName)
	if err != nil {
		fmt.Printf("%v\n", err)
	}

	fileNo := 0
	buf := make([]byte, kCHRSize)
	for {
		bytesRead, err := inFile.Read(buf)
		if err != nil {
			if err != io.EOF {
				fmt.Println(err)
				os.Exit(-1)
			}
			break
		}
		drawPNG(fileNo, buf[:bytesRead])
		fileNo++
	}
}

func setTilePixel(y int, line byte, buf []uint, add bool) {
	mirror := bits.Reverse8(line)
	for x := 0; x < 8; x++ {
		c := uint(mirror) >> uint(x) & 0x1
		pos := y*8 + x
		if add {
			buf[pos] = buf[pos]*2 + c
		} else {
			buf[pos] = c
		}
	}
}

func writeTile(img *image.RGBA, page, tx, ty int, pixels []uint) {
	for y := 0; y < 8; y++ {
		for x := 0; x < 8; x++ {
			pixel := pixels[y*8+x]
			ox := (tx+page*16)*8 + x
			oy := ty*8 + y
			paletteValue := spritePalette[pixel]
			r := palette[paletteValue*kRGBSize]
			g := palette[paletteValue*kRGBSize+1]
			b := palette[paletteValue*kRGBSize+2]
			img.Set(ox, oy, color.RGBA{r, g, b, 255})
		}
	}
}

func drawPNG(number int, data []byte) {
	fn := fmt.Sprintf("%v_%04d.png", outFile, number)
	img := image.NewRGBA(image.Rect(0, 0, 256, 128))

	tileData := make([]uint, 64)
	for i, b := range data {
		page := i / kPageSizeInBytes
		ii := i % kPageSizeInBytes
		tileX := ii / 16 % 16
		tileY := ii / 256
		ti := i % 16
		if ti < 8 {
			// first pass
			setTilePixel(i%8, b, tileData, false)
		} else {
			// second pass
			setTilePixel(i%8, b, tileData, true)
		}
		if ti == 15 {
			// draw
			writeTile(img, page, tileX, tileY, tileData)
		}
	}

	f, err := os.OpenFile(fn, os.O_WRONLY|os.O_CREATE, 0600)
	defer f.Close()
	if err != nil {
		fmt.Println(err)
		os.Exit(-1)
	}

	png.Encode(f, img)
}
