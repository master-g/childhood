package main

import (
	"bytes"
	"fmt"
	"io"
)

const (
	// HeaderSize standard NES rom header is 16 bytes
	HeaderSize int = 16
)

type Header struct {
	Identifier [4]byte // Identifier must be ascii 'NES' and a MS-DOS character break
	PRG        uint8   // PRG size of PRG ROM in 16 KB units
	CHR        uint8   // CHR size of CHR ROM in 8KB units, 0 means CHR RAM only
}

var (
	standardIdentifier = []byte{0x4E, 0x45, 0x53, 0x1A}
)

// NewHeader create a new header from data
func NewHeader(r io.Reader) *Header {
	buf := make([]byte, HeaderSize)
	n, err := r.Read(buf)
	if n != HeaderSize || err != nil {
		return nil
	}
	h := &Header{}
	copy(h.Identifier[:], buf[:4])
	if bytes.Compare(h.Identifier[:], standardIdentifier) != 0 {
		return nil
	}

	h.PRG = buf[4]
	h.CHR = buf[5]

	return h
}

func (h *Header) String() string {
	return fmt.Sprintf("ID:%v PRG:%v CHR:%v", string(h.Identifier[:]), h.PRG, h.CHR)
}
