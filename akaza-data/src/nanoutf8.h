#pragma once

static size_t nanoutf8_byte_count_from_first_char(char c) {
  if ((c & 0x80) == 0x00)
    return 1;
  if ((c & 0xE0) == 0xC0)
    return 2;
  if ((c & 0xF0) == 0xE0)
    return 3;
  if ((c & 0xF8) == 0xF0)
    return 4;
  if ((c & 0xFC) == 0xF8)
    return 5;
  if ((c & 0xFE) == 0xFC)
    return 6;

  return 1;
}
