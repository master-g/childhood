// Copyright © 2019 mg
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

package main

import (
	"fmt"
	"log"
	"strings"

	ui "github.com/gizak/termui/v3"
	"github.com/gizak/termui/v3/widgets"
	"github.com/master-g/childhood/go/mgnes"
)

var (
	cpu           *mgnes.MG6502
	bus           *mgnes.Bus
	disassembly   []*mgnes.Disassembly
	paragraphCpu  *widgets.Paragraph
	paragraphCode *widgets.Paragraph
	paragraphRam0 *widgets.Paragraph
	paragraphRam1 *widgets.Paragraph
)

func renderCpu(p *widgets.Paragraph) {
	sb := &strings.Builder{}
	sb.WriteString("[STATUS:](fg:white)")
	sb.WriteRune(' ')
	sb.WriteString("[N](fg:green)")
	sb.WriteRune(' ')
	sb.WriteString("[V](fg:red)")
	sb.WriteRune(' ')
	sb.WriteString("[U](fg:red)")
	sb.WriteRune(' ')
	sb.WriteString("[B](fg:red)")
	sb.WriteRune(' ')
	sb.WriteString("[D](fg:red)")
	sb.WriteRune(' ')
	sb.WriteString("[I](fg:red)")
	sb.WriteRune(' ')
	sb.WriteString("[N](fg:red)")
	sb.WriteRune(' ')
	sb.WriteString("[C](fg:red)")
	sb.WriteRune('\n')
	sb.WriteString("PC: $0x0000 SP: $0001")
	sb.WriteRune('\n')
	sb.WriteString("A: $0C [AD]")
	sb.WriteRune('\n')
	sb.WriteString("X: $0C [AD]")
	sb.WriteRune('\n')
	sb.WriteString("Y: $0C [AD] ")

	p.Text = sb.String()
}

func renderRam(p *widgets.Paragraph, addr uint16, numRow, numCol int) {
	curAddr := addr
	sb := &strings.Builder{}
	for row := 0; row < numRow; row++ {
		sb.WriteString(fmt.Sprintf("$%04X:", curAddr))
		for col := 0; col < numCol; col++ {
			sb.WriteRune(' ')
			sb.WriteString(fmt.Sprintf("%02X", bus.Read(curAddr, true)))
			curAddr++
		}
		sb.WriteRune('\n')
	}
	p.Text = sb.String()
}

func renderCode(p *widgets.Paragraph) {

}

func draw() {
	renderRam(paragraphRam0, 0x0000, 16, 16)
	renderRam(paragraphRam1, 0x8000, 16, 16)
	renderCpu(paragraphCpu)
	renderCode(paragraphCode)

	ui.Render(paragraphRam0, paragraphRam1, paragraphCpu, paragraphCode, paragraphCode)
}

func loadCpu() {
	// create cpu and bus
	cpu = mgnes.NewMG6502()
	if cpu == nil {
		log.Fatal("could not create 6502")
		return
	}

	bus = mgnes.NewBus()
	cpu.Attach(bus)

	// load bytecode
	codes := []byte{0xA2, 0x0A, 0x8E, 0x00, 0x00, 0xA2, 0x03, 0x8E, 0x01, 0x00, 0xAC, 0x00, 0x00, 0xA9, 0x00, 0x18, 0x6D, 0x01, 0x00, 0x88, 0xD0, 0xFA, 0x8D, 0x02, 0x00, 0xEA, 0xEA, 0xEA}
	var offset uint16
	offset = 0x8000
	for i, b := range codes {
		bus.Write(offset+uint16(i), b)
	}

	// set reset vector
	bus.Write(0xFFFC, 0x00)
	bus.Write(0xFFFD, 0x80)

	// disassembly
	disassembly = cpu.Disassemble(0x0000, 0xFFFF)

	// reset
	cpu.Reset()
}

func initLayout() {
	// Ram
	paragraphRam0 = widgets.NewParagraph()
	paragraphRam0.Title = "RAM Page 0x00"
	paragraphRam0.SetRect(0, 0, 56, 18)

	paragraphRam1 = widgets.NewParagraph()
	paragraphRam1.Title = "RAM Page 0x80"
	paragraphRam1.SetRect(0, 18, 56, 36)

	// CPU
	paragraphCpu = widgets.NewParagraph()
	paragraphCpu.Title = "CPU"
	paragraphCpu.SetRect(56, 0, 56+25, 7)

	// Code
	paragraphCode = widgets.NewParagraph()
	paragraphCode.Title = "Disassembly"
	paragraphCode.SetRect(0, 36, 56, 36+8)
}

func main() {
	if err := ui.Init(); err != nil {
		log.Fatalf("failed to initialize termui: %v", err)
	}
	defer ui.Close()

	initLayout()
	loadCpu()

	draw()

	for e := range ui.PollEvents() {
		if e.Type == ui.KeyboardEvent {
			if e.ID == "q" || e.ID == "<C-c>" {
				break
			} else if e.ID == "<Space>" {
				draw()
			}
		}
	}
}
