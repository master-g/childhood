.POSIX:
.SUFFIXES:

CC		?= cc
RM		?= rm -f
INSTALL		?= install -p
COMMIT		:=
COMMIT_INFO	= commit.h

EXTRA_CFLAGS	?= -O2 -Wall -Wno-restrict -Wno-unknown-warning-option

IS_MACARM := 0

# build on macos(arm) support
ifneq ($(OS),Windows_NT)
     UNAME_S := $(shell uname -s)
     ifeq ($(UNAME_S),Darwin)
         UNAME_P := $(shell uname -p)
         ifneq ($(filter arm%,$(UNAME_P)),)
             EXTRA_CFLAGS += -I$(shell brew --prefix)/include
             IS_MACARM = 1
         endif
     endif
endif

CFLAGS		+= ${EXTRA_CFLAGS} -I.

EXEDIR		?= ..
PREFIX		?= /usr
BINDIR		?= $(PREFIX)/bin

ifeq ($(OS),Windows_NT)
  PROGRAM  = $(EXEDIR)/nesasm.exe
else
  PROGRAM  = $(EXEDIR)/nesasm
endif

ifeq ($(COMMIT),)
  COMMIT = $(shell git rev-parse --short HEAD)
else
  COMMIT += $(shell git diff-index HEAD) (dirty)
endif

ifeq ($(OS),Windows_NT)
     LDFLAGS += -largp
     $(PROGRAM) = $(EXEDIR)/$(PROGRAM).exe
else
     UNAME_S := $(shell uname -s)
           ifeq ($(UNAME_S),Darwin)
             LDFLAGS += -largp

             # build on macos(arm) support
             ifeq ($(IS_MACARM), 1)
                 LDFLAGS += -L$(shell brew --prefix)/lib
             endif
           endif
endif

# Object files

OBJS     = main.o input.o assemble.o expr.o code.o command.o\
         macro.o func.o proc.o symbol.o output.o crc.o\
         nes.o

# All *.c file compiled to *.o
.SUFFIXES: .c .o
.c.o:
	$(CC) $(CFLAGS) -o $@ -c $<

# default target
all: commit_file $(PROGRAM)

commit_file: 
	@printf "#define COMMIT \"$(COMMIT)\"\\n" > $(COMMIT_INFO)

clean:

ifeq ($(OS),Windows_NT)
	@if [ -f "$(PROGRAM)" ]; then for f in `cygcheck "$(PROGRAM)" | grep .dll | grep msys` ; do rm -f $(EXEDIR)/`basename "$$f"` ; done fi
endif

	$(RM) $(OBJS) $(PROGRAM) $(COMMIT_INFO)

# Link objects into executable file

$(PROGRAM): $(OBJS)
	$(CC) -o $(PROGRAM) $(OBJS) $(LDFLAGS) $(LDADD)

ifeq ($(OS),Windows_NT)
	@for f in `cygcheck "$(PROGRAM)" | grep .dll | grep msys` ; do cp -vf `cygpath "$$f"` $(EXEDIR)/ ; done
endif

.PHONY: all commit_file clean

