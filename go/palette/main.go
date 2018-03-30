package main

import (
	"encoding/hex"
	"fmt"
	"image"
	"image/color"
	"image/png"
	"os"
	"regexp"
)

const (
	// PaletteLen must be 16 bytes
	PaletteLen = 16
)

func main() {
	if len(os.Args) < 2 {
		fmt.Println("usage: palette data output")
		os.Exit(0)
	}
	output := "out.png"
	if len(os.Args) >= 3 {
		output = os.Args[2]
	}
	raw := os.Args[1]
	reg, _ := regexp.Compile("[^a-fA-F0-9]+")
	after := reg.ReplaceAllString(raw, "")
	decoded, _ := hex.DecodeString(after)
	if len(decoded) < PaletteLen {
		fmt.Println(fmt.Sprintf("palette data must be 16 bytes, got %v bytes\n", len(decoded)))
		os.Exit(0)
	}

	img := image.NewRGBA(image.Rect(0, 0, 256, 240))

	palette := []color.RGBA{
		{0, 0, 0, 0},
		{255, 0, 0, 255},
		{0, 255, 0, 255},
		{0, 0, 255, 255},
	}
	pixels := make([]int, 64)
	for i := 0; i < 8; i++ {
		line1 := decoded[i]
		line2 := decoded[8+i]
		for j := 0; j < 8; j++ {
			b1 := line1 >> uint(j) & (0x01)
			b2 := line2 >> uint(j) & (0x01)
			pixels[i*8+j] = int(b1 + b2)
		}
	}

	for y := 0; y < 8; y++ {
		for x := 0; x < 8; x++ {
			p := pixels[y*8+x]
			img.Set(x, y, palette[p])
		}
	}

	f, _ := os.OpenFile(output, os.O_WRONLY|os.O_CREATE, 0600)
	defer f.Close()
	png.Encode(f, img)
}
