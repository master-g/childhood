#include <stdio.h>
#include <string.h>
#include "defs.h"
#include "externs.h"
#include "protos.h"
#include "nes.h"

/* locals */
static int ines_prg = 0;    /* size of PRG */
static int ines_chr = 0;    /* size of CHR */
static int ines_mapper = 0;  /* ROM mapper type */
static int ines_submapper = 0;  /* ROM submapper type */
static int ines_mirroring = 0;    /* ROM mirroring type */
static int ines_prg_ram = 0;    /* size of PRG RAM */
static int ines_prg_nvram = 0;    /* size of PRG NVRAM */
static int ines_chr_ram = 0;    /* size of CHR RAM */
static int ines_chr_nvram = 0;    /* size of CHR NVRAM */
static int ines_battery = 0;    /* non-volatile memory flag */
static int ines_timing = 0;    /* CPU/PPU timing */

static struct INES20 {    /* INES rom header */
  unsigned char id[4];
  unsigned char prg_size_lsb;
  unsigned char chr_size_lsb;
  unsigned char flags6;
  unsigned char flags7;
  unsigned char mapper_msb_submapper;
  unsigned char prg_chr_size_msb;
  unsigned char prg_ram_size;
  unsigned char chr_ram_size;
  unsigned char timing;
  unsigned char system_console_type;
  unsigned char misc_roms;
  unsigned char exp_device;
} header;


/* ----
 * write_header()
 * ----
 * generate and write rom header
 */

void
nes_write_header(FILE *f, int banks)
{
  /* setup INES header */
  memset(&header, 0, sizeof(header));
  header.id[0] = 'N';
  header.id[1] = 'E';
  header.id[2] = 'S';
  header.id[3] = 26;
  header.prg_size_lsb = ines_prg & 0xFF;
  header.chr_size_lsb = ines_chr & 0xFF;
  switch (ines_mirroring)
  {
    default:
    case 0: /* Horizontal  or mapper-controlled */
      header.flags6 |= 0;
      break;
    case 1: /* Vertical */
      header.flags6 |= 1;
      break;
    case 2: /* Hard-wired four-screen mode */
    case 3:
    case 4:
      header.flags6 |= 8;
      break;
  }
  if (ines_prg_nvram || ines_chr_nvram)
    ines_battery = 1;
  if (ines_battery)
    header.flags6 |= 2;
  header.flags6 |= (ines_mapper & 0x0F) << 4; /* Mapper Number D0..D3 */
  // TODO: Console Type
  header.flags7 |= 8; /* NES 2.0 identifier */
  header.flags7 |= (ines_mapper & 0xF0); /* Mapper Number D4..D7 */
  header.mapper_msb_submapper |= (ines_mapper & 0xF00) >> 8;
  header.mapper_msb_submapper |= ines_submapper << 4;
  header.prg_chr_size_msb |= (ines_prg & 0xF00) >> 8;
  header.prg_chr_size_msb |= (ines_chr & 0xF00) >> 4;
  if (ines_battery && !ines_prg_ram && !ines_prg_nvram) /* for backward compatibility */
    ines_prg_nvram = 7;
  header.prg_ram_size |= ines_prg_ram & 0x0F;
  header.prg_ram_size |= (ines_prg_nvram & 0x0F) << 4;
  if (!ines_chr && !ines_chr_ram) /* for backward compatibility */
    ines_chr_ram = 7;
  header.chr_ram_size |= ines_chr_ram & 0x0F;
  header.chr_ram_size |= (ines_chr_nvram & 0x0F) << 4;
  header.timing = ines_timing;
  // TODO: System Type
  // TODO: Miscellaneous ROMs
  // TODO: Default Expansion Device

  /* write */
  fwrite(&header, sizeof(header), 1, f);
}


/* ----
 * pack_8x8_tile()
 * ----
 * encode a 8x8 tile for the NES
 */

int
nes_pack_8x8_tile(unsigned char *buffer, void *data, int line_offset, int format)
{
  int i, j;
  int cnt, err;
  unsigned int   pixel;
  unsigned char *ptr;
  unsigned int  *packed;

  /* pack the tile only in the last pass */
  if (pass != LAST_PASS)
    return (16);

  /* clear buffer */
  memset(buffer, 0, 16);

  /* encode the tile */
  switch (format) {
  case CHUNKY_TILE:
    /* 8-bit chunky format */
    cnt = 0;
    ptr = data;

    for (i = 0; i < 8; i++) {
      for (j = 0; j < 8; j++) {
        pixel = ptr[j ^ 0x07];
        buffer[cnt]   |= (pixel & 0x01) ? (1 << j) : 0;
        buffer[cnt+8] |= (pixel & 0x02) ? (1 << j) : 0;
      }        
      ptr += line_offset;
      cnt += 1;
    }
    break;

  case PACKED_TILE:
    /* 4-bit packed format */
    cnt = 0;
    err = 0;
    packed = data;
  
    for (i = 0; i < 8; i++) {
      pixel = packed[i];
  
      for (j = 0; j < 8; j++) {
        /* check for errors */
        if (pixel & 0x0C)
          err++;

        /* convert the tile */
        buffer[cnt]   |= (pixel & 0x01) ? (1 << j) : 0;
        buffer[cnt+8] |= (pixel & 0x02) ? (1 << j) : 0;
        pixel >>= 4;
      }
      cnt += 1;
    }

    /* error message */
    if (err)
      error("Incorrect pixel color index!");
    break;

  default:
    /* other formats not supported */
    error("Internal error: unsupported format passed to 'pack_8x8_tile'!");
    break;
  }

  /* ok */
  return (16);
}


/* ----
 * do_defchr()
 * ----
 * .defchr pseudo
 */

void
nes_defchr(int *ip)
{
  unsigned char buffer[16];
  unsigned int data[8];
  int size;
  int i;

  /* define label */
  labldef(loccnt, 1);

  /* output infos */
  data_loccnt = loccnt;
  data_size   = 3;
  data_level  = 3;

  /* get tile data */
  for (i = 0; i < 8; i++) {
    /* get value */
    if (!evaluate(ip, (i < 7) ? ',' : ';'))
      return;

    /* store value */
    data[i] = value;
  }

  /* encode tile */
  size = nes_pack_8x8_tile(buffer, data, 0, PACKED_TILE);

  /* store tile */
  putbuffer(buffer, size);

  /* output line */
  if (pass == LAST_PASS)
    println();
}


/* ----
 * do_inesprg()
 * ----
 * .inesprg pseudo
 */

void
nes_inesprg(int *ip)
{
  if (!evaluate(ip, ';'))
    return;

  if ((value < 0) || (value > 0xEFF * 0x4000)) 
  {
    error("PRG size value out of range!");
  
    return;
  } else if (value > 0xEFF)
  {
    if ((value % 0x4000) != 0)
    {
      error("Invalid PRG size value!");

      return;
    }
    value /= 0x4000;
  }

  ines_prg = value;

  if (pass == LAST_PASS) 
  {
    println();
  }
}


/* ----
 * do_ineschr()
 * ----
 * .ineschr pseudo
 */

void
nes_ineschr(int *ip)
{
  if (!evaluate(ip, ';'))
    return;

  if ((value < 0) || (value > 0xEFF * 0x2000)) 
  {
    error("CHR size value out of range!");
  
    return;
  } else if (value > 0xEFF)
  {
    if ((value % 0x2000) != 0)
    {
      error("Invalid CHR size value!");

      return;
    }
    value /= 0x2000;
  }
  
  ines_chr = value;

  if (pass == LAST_PASS) 
  {
    println();
  }
}

/* ----
 * do_inesprgram()
 * ----
 * .inesprgram pseudo
 */

void
nes_inesprgram(int *ip)
{
  if (!evaluate(ip, ';'))
    return;

  if ((value < 0) || (value > 0x200000)) 
  {
    error("PRG RAM value out of range!");
  
    return;
  } else if (value > 15)
  {
    unsigned char shift = 0;
    while (((64 << shift) != value) && (shift < 16)) shift++;
    if (shift >= 16)
    {
      error("Invalid PRG RAM value!");

      return;
    }
    value = shift;
  }

  ines_prg_ram = value;

  if (pass == LAST_PASS) 
  {
    println();
  }
}


/* ----
 * do_inesprgnvram()
 * ----
 * .inesprgnvram pseudo
 */

void
nes_inesprgnvram(int *ip)
{
  if (!evaluate(ip, ';'))
    return;

  if ((value < 0) || (value > 0x200000)) 
  {
    error("PRG NVRAM value out of range!");
  
    return;
  } else if (value > 15)
  {
    unsigned char shift = 0;
    while (((64 << shift) != value) && (shift < 16)) shift++;
    if (shift >= 16)
    {
      error("Invalid PRG NVRAM value!");

      return;
    }
    value = shift;
  }

  ines_prg_nvram = value;
  if (value) ines_battery = 1;

  if (pass == LAST_PASS) 
  {
    println();
  }
}


/* ----
 * do_ineschrram()
 * ----
 * .ineschrram pseudo
 */

void
nes_ineschrram(int *ip)
{
  if (!evaluate(ip, ';'))
    return;

  if ((value < 0) || (value > 0x200000)) 
  {
    error("CHR RAM value out of range!");
  
    return;
  } else if (value > 15)
  {
    unsigned char shift = 0;
    while (((64 << shift) != value) && (shift < 16)) shift++;
    if (shift >= 16)
    {
      error("Invalid CHR RAM value!");

      return;
    }
    value = shift;
  }

  ines_chr_ram = value;

  if (pass == LAST_PASS) 
  {
    println();
  }
}


/* ----
 * do_ineschrnvram()
 * ----
 * .ineschrnvram pseudo
 */

void
nes_ineschrnvram(int *ip)
{
  if (!evaluate(ip, ';'))
    return;

  if ((value < 0) || (value > 0x200000)) 
  {
    error("CHR NVRAM value out of range!");
  
    return;
  } else if (value > 15)
  {
    unsigned char shift = 0;
    while (((64 << shift) != value) && (shift < 16)) shift++;
    if (shift >= 16)
    {
      error("Invalid CHR NVRAM value!");

      return;
    }
    value = shift;
  }

  ines_chr_nvram = value;
  if (value) ines_battery = 1;

  if (pass == LAST_PASS) 
  {
    println();
  }
}


/* ----
 * do_inesmap()
 * ----
 * .inesmap pseudo
 */

void
nes_inesmap(int *ip)
{
  if (!evaluate(ip, ';'))
    return;

  if ((value < 0) || (value > 4095)) 
  {
    error("Mapper value out of range!");
  
    return;
  }
  
  ines_mapper = value;

  if (pass == LAST_PASS) 
  {
    println();
  }
}


/* ----
 * do_inessubmap()
 * ----
 * .inessubmap pseudo
 */

void
nes_inessubmap(int *ip)
{
  if (!evaluate(ip, ';'))
    return;

  if ((value < 0) || (value > 15)) 
  {
    error("Submapper value out of range!");
  
    return;
  }
  
  ines_submapper = value;

  if (pass == LAST_PASS) 
  {
    println();
  }
}


/* ----
 * do_inesmir()
 * ----
 * .inesmir pseudo
 */

void
nes_inesmir(int *ip)
{
  if (!evaluate(ip, ';'))
    return;

  if ((value < 0) || (value > 4)) 
  {
    error("Mirror value out of range!");
  
    return;
  }
  
  ines_mirroring = value;

  if (pass == LAST_PASS) 
  {
    println();
  }
}


/* ----
 * do_inesbat()
 * ----
 * .inesbat pseudo
 */

void
nes_inesbat(int *ip)
{
  if (!evaluate(ip, ';'))
    return;

  if ((value < 0) || (value > 1)) 
  {
    error("Battery value out of range!");
  
    return;
  }
  
  ines_battery = value;

  if (pass == LAST_PASS) 
  {
    println();
  }
}


/* ----
 * do_inestim()
 * ----
 * .inestim pseudo
 */

void
nes_inestim(int *ip)
{
  if (!evaluate(ip, ';'))
    return;

  if ((value < 0) || (value > 3)) 
  {
    error("Timing value out of range!");
  
    return;
  }
  
  ines_timing = value;

  if (pass == LAST_PASS) 
  {
    println();
  }
}
