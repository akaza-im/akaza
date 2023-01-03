#include <ibus.h>

#pragma once

typedef gboolean (*ibus_akaza_callback_key_event)(IBusEngine* engine, guint keyval, guint keycode, guint modifiers);
void ibus_akaza_set_callback(ibus_akaza_callback_key_event* cb);

typedef enum {
    ALNUM,
    HIRAGANA,
    // TODO support more input modes.
} InputMode;

typedef struct {
  IBusEngine parent;

  /* members */
  GString *preedit;
  gint cursor_pos;

  IBusLookupTable *table;

  InputMode input_mode;
} IBusAkazaEngine;
