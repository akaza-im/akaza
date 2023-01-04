#include <ibus.h>

#pragma once

typedef gboolean (*ibus_akaza_callback_key_event)(void* ctx, IBusEngine* engine, guint keyval, guint keycode, guint modifiers);
void ibus_akaza_set_callback(void* ctx, ibus_akaza_callback_key_event* cb);

// TODO deprecate this
typedef struct {
  IBusEngine parent;
} IBusAkazaEngine;
