#include <ibus.h>
#include <string.h>
#include <enchant.h>
#include <stdbool.h>
#include "config.h"


#define IBUS_TYPE_ENCHANT_ENGINE        \
        (ibus_akaza_engine_get_type ())

GType   ibus_akaza_engine_get_type    (void);



typedef struct _IBusAkazaEngine IBusAkazaEngine;
typedef struct _IBusAkazaEngineClass IBusAkazaEngineClass;

struct _IBusAkazaEngine {
  IBusEngine parent;

  /* members */
  GString *preedit;
  gint cursor_pos;

  IBusLookupTable *table;
};

struct _IBusAkazaEngineClass {
  IBusEngineClass parent;
};

/* functions prototype */
static void ibus_akaza_engine_class_init(IBusAkazaEngineClass *klass);
static void ibus_akaza_engine_init(IBusAkazaEngine *engine);
static void ibus_akaza_engine_destroy(IBusAkazaEngine *engine);
static gboolean ibus_akaza_engine_process_key_event(IBusEngine *engine,
                                                      guint keyval,
                                                      guint keycode,
                                                      guint modifiers);
static void ibus_akaza_engine_commit_string(IBusAkazaEngine *enchant,
                                              const gchar *string);
static void ibus_akaza_engine_update(IBusAkazaEngine *enchant);

static EnchantBroker *broker = NULL;
static EnchantDict *dict = NULL;

G_DEFINE_TYPE(IBusAkazaEngine, ibus_akaza_engine, IBUS_TYPE_ENGINE)

static void ibus_akaza_engine_class_init(IBusAkazaEngineClass *klass) {
  IBusObjectClass *ibus_object_class = IBUS_OBJECT_CLASS(klass);
  IBusEngineClass *engine_class = IBUS_ENGINE_CLASS(klass);

  ibus_object_class->destroy =
      (IBusObjectDestroyFunc)ibus_akaza_engine_destroy;

  engine_class->process_key_event = ibus_akaza_engine_process_key_event;
}

static void ibus_akaza_engine_init(IBusAkazaEngine *enchant) {
  if (broker == NULL) {
    broker = enchant_broker_init();
    dict = enchant_broker_request_dict(broker, "en");
  }

  enchant->preedit = g_string_new("");
  enchant->cursor_pos = 0;

  enchant->table = ibus_lookup_table_new(9, 0, TRUE, TRUE);
  g_object_ref_sink(enchant->table);
}

static void ibus_akaza_engine_destroy(IBusAkazaEngine *enchant) {
  if (enchant->preedit) {
    g_string_free(enchant->preedit, TRUE);
    enchant->preedit = NULL;
  }

  if (enchant->table) {
    g_object_unref(enchant->table);
    enchant->table = NULL;
  }

  ((IBusObjectClass *)ibus_akaza_engine_parent_class)
      ->destroy((IBusObject *)enchant);
}

static void ibus_akaza_engine_update_lookup_table(
    IBusAkazaEngine *enchant) {
  gchar **sugs;
  gint n_sug, i;

  if (enchant->preedit->len == 0) {
    ibus_engine_hide_lookup_table((IBusEngine *)enchant);
    return;
  }

  ibus_lookup_table_clear(enchant->table);

  sugs = enchant_dict_suggest(dict, enchant->preedit->str,
                              enchant->preedit->len, &n_sug);

  if (sugs == NULL || n_sug == 0) {
    ibus_engine_hide_lookup_table((IBusEngine *)enchant);
    return;
  }

  for (i = 0; i < n_sug; i++) {
    ibus_lookup_table_append_candidate(enchant->table,
                                       ibus_text_new_from_string(sugs[i]));
  }

  ibus_engine_update_lookup_table((IBusEngine *)enchant, enchant->table, TRUE);

//  if (sugs) enchant_dict_free_suggestions(dict, sugs);
}

static void ibus_akaza_engine_update_preedit(IBusAkazaEngine *enchant) {
  IBusText *text;
  gint retval;

  text = ibus_text_new_from_static_string(enchant->preedit->str);
  text->attrs = ibus_attr_list_new();

  ibus_attr_list_append(text->attrs,
                        ibus_attr_underline_new(IBUS_ATTR_UNDERLINE_SINGLE, 0,
                                                enchant->preedit->len));

  if (enchant->preedit->len > 0) {
    retval =
        enchant_dict_check(dict, enchant->preedit->str, enchant->preedit->len);
    if (retval != 0) {
      ibus_attr_list_append(
          text->attrs,
          ibus_attr_foreground_new(0xff0000, 0, enchant->preedit->len));
    }
  }

  ibus_engine_update_preedit_text((IBusEngine *)enchant, text,
                                  enchant->cursor_pos, TRUE);
}

/* commit preedit to client and update preedit */
static gboolean ibus_akaza_engine_commit_preedit(IBusAkazaEngine *enchant) {
  if (enchant->preedit->len == 0) return FALSE;

  ibus_akaza_engine_commit_string(enchant, enchant->preedit->str);
  g_string_assign(enchant->preedit, "");
  enchant->cursor_pos = 0;

  ibus_akaza_engine_update(enchant);

  return TRUE;
}

static void ibus_akaza_engine_commit_string(IBusAkazaEngine *enchant,
                                              const gchar *string) {
  IBusText *text;
  text = ibus_text_new_from_static_string(string);
  ibus_engine_commit_text((IBusEngine *)enchant, text);
}

static void ibus_akaza_engine_update(IBusAkazaEngine *enchant) {
  ibus_akaza_engine_update_preedit(enchant);
  ibus_engine_hide_lookup_table((IBusEngine *)enchant);
}

#define is_alpha(c) \
  (((c) >= IBUS_a && (c) <= IBUS_z) || ((c) >= IBUS_A && (c) <= IBUS_Z))

static gboolean ibus_akaza_engine_process_key_event(IBusEngine *engine,
                                                      guint keyval,
                                                      guint keycode,
                                                      guint modifiers) {
  IBusAkazaEngine *enchant = (IBusAkazaEngine *)engine;

  if (modifiers & IBUS_RELEASE_MASK) return FALSE;

  modifiers &= (IBUS_CONTROL_MASK | IBUS_MOD1_MASK);

  if (modifiers == IBUS_CONTROL_MASK && keyval == IBUS_s) {
    ibus_akaza_engine_update_lookup_table(enchant);
    return TRUE;
  }

  if (modifiers != 0) {
    if (enchant->preedit->len == 0)
      return FALSE;
    else
      return TRUE;
  }

  switch (keyval) {
    case IBUS_space:
      g_string_append(enchant->preedit, " ");
      return ibus_akaza_engine_commit_preedit(enchant);
    case IBUS_Return:
      return ibus_akaza_engine_commit_preedit(enchant);

    case IBUS_Escape:
      if (enchant->preedit->len == 0) return FALSE;

      g_string_assign(enchant->preedit, "");
      enchant->cursor_pos = 0;
      ibus_akaza_engine_update(enchant);
      return TRUE;

    case IBUS_Left:
      if (enchant->preedit->len == 0) return FALSE;
      if (enchant->cursor_pos > 0) {
        enchant->cursor_pos--;
        ibus_akaza_engine_update(enchant);
      }
      return TRUE;

    case IBUS_Right:
      if (enchant->preedit->len == 0) return FALSE;
      if (enchant->cursor_pos < enchant->preedit->len) {
        enchant->cursor_pos++;
        ibus_akaza_engine_update(enchant);
      }
      return TRUE;

    case IBUS_Up:
      if (enchant->preedit->len == 0) return FALSE;
      if (enchant->cursor_pos != 0) {
        enchant->cursor_pos = 0;
        ibus_akaza_engine_update(enchant);
      }
      return TRUE;

    case IBUS_Down:
      if (enchant->preedit->len == 0) return FALSE;

      if (enchant->cursor_pos != enchant->preedit->len) {
        enchant->cursor_pos = enchant->preedit->len;
        ibus_akaza_engine_update(enchant);
      }

      return TRUE;

    case IBUS_BackSpace:
      if (enchant->preedit->len == 0) return FALSE;
      if (enchant->cursor_pos > 0) {
        enchant->cursor_pos--;
        g_string_erase(enchant->preedit, enchant->cursor_pos, 1);
        ibus_akaza_engine_update(enchant);
      }
      return TRUE;

    case IBUS_Delete:
      if (enchant->preedit->len == 0) return FALSE;
      if (enchant->cursor_pos < enchant->preedit->len) {
        g_string_erase(enchant->preedit, enchant->cursor_pos, 1);
        ibus_akaza_engine_update(enchant);
      }
      return TRUE;
  }

  if (is_alpha(keyval)) {
    g_string_insert_c(enchant->preedit, enchant->cursor_pos, keyval);

    enchant->cursor_pos++;
    ibus_akaza_engine_update(enchant);

    return TRUE;
  }

  return FALSE;
}

static IBusBus *bus = NULL;

static void ibus_disconnected_cb(IBusBus *bus, gpointer user_data) {
  ibus_quit();
}

void tmp_akaza_init(bool ibus) {
  ibus_init();

  bus = ibus_bus_new();
  g_object_ref_sink(bus);
  g_signal_connect(bus, "disconnected", G_CALLBACK(ibus_disconnected_cb), NULL);

  IBusFactory * factory = ibus_factory_new(ibus_bus_get_connection(bus));
  g_object_ref_sink(factory);
  ibus_factory_add_engine(factory, "akaza", IBUS_TYPE_ENCHANT_ENGINE);

  if (ibus) {
    ibus_bus_request_name(bus, "org.freedesktop.IBus.Akaza", 0);
  } else {
    IBusComponent *component;

    component =
        ibus_component_new("org.freedesktop.IBus.Akaza", "Akaza", "0.1.0",
                           "GPL", "Tokuhiro Matsuno <tokuhirom@gmail.com>",
                           "https://github.com/tokuhirom/akaza/", "", "ibus-akaza");
    ibus_component_add_engine(
        component,
        ibus_engine_desc_new("akaza", "Akaza", "Akaza", "ja", "MIT",
                             "tokuhirom <tokuhirom@gmail.com>",
                             PKGDATADIR "/icons/ibus-akaza.svg", "us"));
    ibus_bus_register_component(bus, component);
  }
}

