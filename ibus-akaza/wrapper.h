#include <ibus.h>

#pragma once

typedef gboolean (*ibus_akaza_callback_key_event)(void* ctx, IBusEngine* engine, guint keyval, guint keycode, guint modifiers);
typedef gboolean (*ibus_akaza_callback_candidate_clicked)(void* ctx, IBusEngine* engine, guint index, guint button, guint state);
void ibus_akaza_set_callback(void* ctx, ibus_akaza_callback_key_event* cb, ibus_akaza_callback_candidate_clicked*);

typedef struct {
  IBusEngine parent;
} IBusAkazaEngine;
