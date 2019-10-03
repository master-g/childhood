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

package bus

import (
	"mgnes/pkg/cartridge"
	"mgnes/pkg/log"
	"mgnes/pkg/memory"
	"mgnes/pkg/mg2c02"
	"mgnes/pkg/mg6502"
)

// Bus transmit data between cpu and other components in the NES console
type Bus struct {
	cpu  *mg6502.MG6502
	ppu  *mg2c02.MG2C02
	cart *cartridge.Cartridge
	ram  memory.Memory

	systemClockCounter int
}

// NewBus create and return a new bus reference
func NewBus(cpu *mg6502.MG6502) (bus *Bus) {
	if cpu == nil {
		log.L("invalid cpu")
		return
	}
	bus = &Bus{
		cpu:  cpu,
		ppu:  nil,
		ram:  memory.NewCpuMemory(),
		cart: nil,
	}
	cpu.SetReader(bus)
	cpu.SetWriter(bus)

	return
}

// CpuWrite writes data to the bus
func (bus *Bus) CpuWrite(addr uint16, data uint8) {
	if bus.cart.CpuWrite(addr, data) {
		// The cartridge "sees all" and has the facility to veto
		// the propagation of the bus transaction if it requires.
		// This allows the cartridge to map any address to some
		// other data, including the facility to divert transactions
		// with other physical devices. The NES does not do this
	} else if addr <= 0x1FFF {
		// System RAM Address Range. The range covers 8KB, though
		// there is only 2KB available. That 2KB is "mirrored"
		// through this address range. Using bitwise AND to mask
		// the bottom 11 bits is the same as addr % 2048.
		bus.ram.Write(addr, data)
	} else if addr >= 0x2000 && addr <= 0x3FFF {
		// PPU Address range. The PPU only has 8 primary registers
		// and these are repeated throughout this range. We can
		// use bitwise AND operation to mask the bottom 3 bits,
		// which is the equivalent of addr % 8.
		bus.ppu.CpuWrite(addr, data)
	}
}

// CpuRead data from the bus
func (bus *Bus) CpuRead(addr uint16, readonly bool) (data uint8) {
	flag := false
	if data, flag = bus.cart.CpuRead(addr); flag {
		// cartridge address range
	} else if addr <= 0x1FFF {
		// system RAM address range, mirrored every 2048 bytes
		data = bus.ram.Read(addr)
	} else if addr >= 0x2000 && addr <= 0x3FFF {
		// PPU address range, mirrored every 8 bytes
		data = bus.ppu.CpuRead(addr, readonly)
	}
	return
}

// InsertCartridge attach a cartridge to the bus
func (bus *Bus) InsertCartridge(cart *cartridge.Cartridge) {
	bus.cart = cart
	bus.ppu.AttachCartridge(cart)
}

// Reset sends a reset signal to all components attached to this bus
func (bus *Bus) Reset() {
	bus.cpu.Reset()
	bus.systemClockCounter = 0
}

// Clock ticks the whole system
func (bus *Bus) Clock() {
	// Clocking. The heart and soul of an emulator. The running
	// frequency is controlled by whatever calls this function.
	// So here we "divide" the clock as necessary and call
	// the peripheral devices clock() function at the correct
	// times.

	// The fastest clock frequency the digital system cares
	// about is equivalent to the PPU clock. So the PPU is clocked
	// each time this function is called.
	bus.ppu.Clock()

	// The CPU runs 3 times slower than the PPU so we only call its
	// clock() function every 3 times this function is called. We
	// have a global counter to keep track of this.
	if bus.systemClockCounter%3 == 0 {
		bus.cpu.Clock()
	}

	bus.systemClockCounter++
}
