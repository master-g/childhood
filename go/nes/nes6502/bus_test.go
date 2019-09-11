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

package nes6502

import (
	"testing"
)

func TestBus_Read(t *testing.T) {
	bus := NewBus()
	vec1 := bus.Read(0, true)
	if vec1 != 0 {
		t.Errorf("Read() = %v, want 0", vec1)
	}

	bus.Write(1, 0xDE)
	vec2 := bus.Read(1, true)
	if vec2 != 0xDE {
		t.Errorf("Read() = %v, want 0xDE", vec2)
	}

	bus.Write(2, 0xAD)
	vec3 := bus.Read(2, true)
	if vec3 != 0xAD {
		t.Errorf("Read() = %v, want 0xAD", vec3)
	}

	bus.Write(MemoryCapacity-1, 0x22)
	vec4 := bus.Read(MemoryCapacity-1, true)
	if vec4 != 0x22 {
		t.Errorf("Read() = %v, want 0x22", vec4)
	}
}
