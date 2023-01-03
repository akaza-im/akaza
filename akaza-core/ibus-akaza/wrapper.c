#include <ibus.h>
#include <string.h>
#include <enchant.h>
#include <stdbool.h>
#include <stdio.h>
#include "config.h"
#include "wrapper.h"

// for debugging
// TODO remove this? or configurable path?
void akaza_log(const char*format, ...) {
    va_list ap;

    // 可変長引数を１個の変数にまとめる
    va_start( ap, format );
    // まとめられた変数で処理する
    vprintf( format, ap );
    va_end( ap );

    FILE* fp = fopen("/tmp/ibus-akaza-rust-wrapper.log", "a");
    if (fp != NULL) {
        vfprintf(fp, format, ap);
    }
    fclose(fp);
}

// Callback for key typed.
static ibus_akaza_callback_key_event global_key_event_cb;

#define IBUS_TYPE_AKAZA_ENGINE        \
        (ibus_akaza_engine_get_type ())

GType   ibus_akaza_engine_get_type    (void);



typedef struct _IBusAkazaEngineClass IBusAkazaEngineClass;


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
static void ibus_akaza_engine_commit_string(IBusAkazaEngine *akaza,
                                              const gchar *string);
static void ibus_akaza_engine_update(IBusAkazaEngine *akaza);

G_DEFINE_TYPE(IBusAkazaEngine, ibus_akaza_engine, IBUS_TYPE_ENGINE)

static void ibus_akaza_engine_class_init(IBusAkazaEngineClass *klass) {
  IBusObjectClass *ibus_object_class = IBUS_OBJECT_CLASS(klass);
  IBusEngineClass *engine_class = IBUS_ENGINE_CLASS(klass);

  ibus_object_class->destroy =
      (IBusObjectDestroyFunc)ibus_akaza_engine_destroy;

  engine_class->process_key_event = ibus_akaza_engine_process_key_event;
}

static void ibus_akaza_engine_init(IBusAkazaEngine *akaza) {
//  if (broker == NULL) {
//    broker = enchant_broker_init();
//    dict = enchant_broker_request_dict(broker, "en");
//  }

  akaza->preedit = g_string_new("");
  akaza->cursor_pos = 0;
  akaza->input_mode = HIRAGANA;

  akaza->table = ibus_lookup_table_new(9, 0, TRUE, TRUE);
  g_object_ref_sink(akaza->table);
}

static void ibus_akaza_engine_destroy(IBusAkazaEngine *akaza) {
  if (akaza->preedit) {
    g_string_free(akaza->preedit, TRUE);
    akaza->preedit = NULL;
  }

  if (akaza->table) {
    g_object_unref(akaza->table);
    akaza->table = NULL;
  }

  ((IBusObjectClass *)ibus_akaza_engine_parent_class)
      ->destroy((IBusObject *)akaza);
}

static void ibus_akaza_engine_update_lookup_table(
    IBusAkazaEngine *akaza) {
  gchar **sugs;
  gint n_sug, i;

  if (akaza->preedit->len == 0) {
    ibus_engine_hide_lookup_table((IBusEngine *)akaza);
    return;
  }

  ibus_lookup_table_clear(akaza->table);

  // XXX i need to implement kana-kanji conversion here.
//  sugs = enchant_dict_suggest(dict, akaza->preedit->str,
//                              akaza->preedit->len, &n_sug);
//
sugs = NULL;
  if (sugs == NULL || n_sug == 0) {
    ibus_engine_hide_lookup_table((IBusEngine *)akaza);
    return;
  }

  for (i = 0; i < n_sug; i++) {
    ibus_lookup_table_append_candidate(akaza->table,
                                       ibus_text_new_from_string(sugs[i]));
  }

  ibus_engine_update_lookup_table((IBusEngine *)akaza, akaza->table, TRUE);

//  if (sugs) enchant_dict_free_suggestions(dict, sugs);
}

static void ibus_akaza_engine_update_preedit(IBusAkazaEngine *akaza) {
  IBusText *text;
  gint retval;

  text = ibus_text_new_from_static_string(akaza->preedit->str);
  text->attrs = ibus_attr_list_new();

  ibus_attr_list_append(text->attrs,
                        ibus_attr_underline_new(IBUS_ATTR_UNDERLINE_SINGLE, 0,
                                                akaza->preedit->len));

  if (akaza->preedit->len > 0) {
  retval = 0;
//    retval =
//        enchant_dict_check(dict, akaza->preedit->str, akaza->preedit->len);
    if (retval != 0) {
      ibus_attr_list_append(
          text->attrs,
          ibus_attr_foreground_new(0xff0000, 0, akaza->preedit->len));
    }
  }

  ibus_engine_update_preedit_text((IBusEngine *)akaza, text,
                                  akaza->cursor_pos, TRUE);
}

/* commit preedit to client and update preedit */
static gboolean ibus_akaza_engine_commit_preedit(IBusAkazaEngine *akaza) {
  if (akaza->preedit->len == 0) return FALSE;

  ibus_akaza_engine_commit_string(akaza, akaza->preedit->str);
  g_string_assign(akaza->preedit, "");
  akaza->cursor_pos = 0;

  ibus_akaza_engine_update(akaza);

  return TRUE;
}

static void ibus_akaza_engine_commit_string(IBusAkazaEngine *akaza,
                                              const gchar *string) {
  IBusText* text = ibus_text_new_from_static_string(string);
  ibus_engine_commit_text((IBusEngine *)akaza, text);
  // [text] will be released by ibus_engine_commit_text.
}

static void ibus_akaza_engine_update(IBusAkazaEngine *akaza) {
  ibus_akaza_engine_update_preedit(akaza);
  ibus_engine_hide_lookup_table((IBusEngine *)akaza);
}

#define is_alpha(c) \
  (((c) >= IBUS_a && (c) <= IBUS_z) || ((c) >= IBUS_A && (c) <= IBUS_Z))

static gboolean ibus_akaza_engine_process_key_event(IBusEngine *engine,
                                                      guint keyval,
                                                      guint keycode,
                                                      guint modifiers) {
  IBusAkazaEngine *akaza = (IBusAkazaEngine *)engine;

  akaza_log("process_key_event(%04x, %04x, %04x)\n", keyval, keycode, modifiers);

  // ignore key release event.
  if (modifiers & IBUS_RELEASE_MASK) return FALSE;

  modifiers &= (IBUS_CONTROL_MASK | IBUS_MOD1_MASK);

  if (modifiers == IBUS_CONTROL_MASK && keyval == IBUS_s) {
    ibus_akaza_engine_update_lookup_table(akaza);
    return TRUE;
  }

  if (modifiers != 0) {
    if (akaza->preedit->len == 0)
      return FALSE;
    else
      return TRUE;
  }

  switch (keyval) {
    case IBUS_space:
      g_string_append(akaza->preedit, " ");
      return ibus_akaza_engine_commit_preedit(akaza);
    case IBUS_Return:
      return ibus_akaza_engine_commit_preedit(akaza);

    case IBUS_Escape:
      if (akaza->preedit->len == 0) return FALSE;

      g_string_assign(akaza->preedit, "");
      akaza->cursor_pos = 0;
      ibus_akaza_engine_update(akaza);
      return TRUE;

    case IBUS_Left:
      if (akaza->preedit->len == 0) return FALSE;
      if (akaza->cursor_pos > 0) {
        akaza->cursor_pos--;
        ibus_akaza_engine_update(akaza);
      }
      return TRUE;

    case IBUS_Right:
      if (akaza->preedit->len == 0) return FALSE;
      if (akaza->cursor_pos < akaza->preedit->len) {
        akaza->cursor_pos++;
        ibus_akaza_engine_update(akaza);
      }
      return TRUE;

    case IBUS_Up:
      if (akaza->preedit->len == 0) return FALSE;
      if (akaza->cursor_pos != 0) {
        akaza->cursor_pos = 0;
        ibus_akaza_engine_update(akaza);
      }
      return TRUE;

    case IBUS_Down:
      if (akaza->preedit->len == 0) return FALSE;

      if (akaza->cursor_pos != akaza->preedit->len) {
        akaza->cursor_pos = akaza->preedit->len;
        ibus_akaza_engine_update(akaza);
      }

      return TRUE;

    case IBUS_BackSpace:
      if (akaza->preedit->len == 0) return FALSE;
      if (akaza->cursor_pos > 0) {
        akaza->cursor_pos--;
        g_string_erase(akaza->preedit, akaza->cursor_pos, 1);
        ibus_akaza_engine_update(akaza);
      }
      return TRUE;

    case IBUS_Delete:
      if (akaza->preedit->len == 0) return FALSE;
      if (akaza->cursor_pos < akaza->preedit->len) {
        g_string_erase(akaza->preedit, akaza->cursor_pos, 1);
        ibus_akaza_engine_update(akaza);
      }
      return TRUE;
  }

  return global_key_event_cb(akaza, keyval, keycode, modifiers);

/*
  if ('!' <= keyval && keyval <= '~') {
    g_string_insert_c(akaza->preedit, akaza->cursor_pos, keyval);

    akaza->cursor_pos++;
    ibus_akaza_engine_update(akaza);

    return TRUE;
  }

  return FALSE;
  */
}

static void ibus_disconnected_cb(IBusBus *bus, gpointer user_data) {
  ibus_quit();
}


void ibus_akaza_set_callback(ibus_akaza_callback_key_event* cb) {
    global_key_event_cb = cb;
}

void ibus_akaza_init(bool ibus) {
  akaza_log("Akaza bootstrap(in wrapper.c)\n");

  ibus_init();

  struct IBusBus* bus = ibus_bus_new();
  g_object_ref_sink(bus);
  g_signal_connect(bus, "disconnected", G_CALLBACK(ibus_disconnected_cb), NULL);

  IBusFactory * factory = ibus_factory_new(ibus_bus_get_connection(bus));
  g_object_ref_sink(factory);
  ibus_factory_add_engine(factory, "akaza", IBUS_TYPE_AKAZA_ENGINE);

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

