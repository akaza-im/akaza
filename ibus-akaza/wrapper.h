#include <ibus.h>

#pragma once

typedef gboolean (*ibus_akaza_callback_key_event)(void* ctx, IBusEngine* engine, guint keyval, guint keycode, guint modifiers);
typedef gboolean (*ibus_akaza_callback_candidate_clicked)(void* ctx, IBusEngine* engine, guint index, guint button, guint state);
typedef void (*ibus_akaza_callback_focus_in)(void* ctx, IBusEngine* engine);
typedef void (*ibus_akaza_callback_property_activate)(void* ctx, IBusEngine* engine, const gchar *prop_name, guint prop_state);

void ibus_akaza_set_callback(void* ctx, ibus_akaza_callback_key_event* cb, ibus_akaza_callback_candidate_clicked*, ibus_akaza_callback_focus_in*, ibus_akaza_callback_property_activate*);

typedef struct {
  IBusEngine parent;
} IBusAkazaEngine;
