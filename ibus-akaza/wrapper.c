#include <ibus.h>
#include <string.h>
#include <stdbool.h>
#include <stdio.h>
#include "config.h"
#include "wrapper.h"

// Callback for key typed.
static void* global_context = NULL;
static ibus_akaza_callback_key_event global_key_event_cb = NULL;
static ibus_akaza_callback_candidate_clicked global_candidate_clicked_cb = NULL;

#define IBUS_TYPE_AKAZA_ENGINE        \
        (ibus_akaza_engine_get_type ())

GType ibus_akaza_engine_get_type(void);

typedef struct  {
  IBusEngineClass parent;
} IBusAkazaEngineClass;

/* functions prototype */
static void ibus_akaza_engine_class_init(IBusAkazaEngineClass *klass);
static void ibus_akaza_engine_init(IBusAkazaEngine *engine);
static void ibus_akaza_engine_destroy(IBusAkazaEngine *engine);
static gboolean ibus_akaza_engine_process_key_event(IBusEngine *engine,
                                                      guint keyval,
                                                      guint keycode,
                                                      guint modifiers);

G_DEFINE_TYPE(IBusAkazaEngine, ibus_akaza_engine, IBUS_TYPE_ENGINE)

static void ibus_akaza_engine_init(IBusAkazaEngine *akaza) {
}

static void ibus_akaza_engine_destroy(IBusAkazaEngine *akaza) {
  ((IBusObjectClass *)ibus_akaza_engine_parent_class)
      ->destroy((IBusObject *)akaza);
}

static gboolean ibus_akaza_engine_candidate_clicked(
    IBusEngine *engine,
    int index,
    int button,
    int state
) {
  return global_candidate_clicked_cb(global_context, engine, index, button, state);
}

static gboolean ibus_akaza_engine_process_key_event(IBusEngine *engine,
                                                      guint keyval,
                                                      guint keycode,
                                                      guint modifiers) {
  return global_key_event_cb(global_context, engine, keyval, keycode, modifiers);
}

static void ibus_disconnected_cb(IBusBus *bus, gpointer user_data) {
  ibus_quit();
}

static void ibus_akaza_engine_class_init(IBusAkazaEngineClass *klass) {
  IBusObjectClass *ibus_object_class = IBUS_OBJECT_CLASS(klass);
  IBusEngineClass *engine_class = IBUS_ENGINE_CLASS(klass);

  ibus_object_class->destroy =
      (IBusObjectDestroyFunc)ibus_akaza_engine_destroy;

  engine_class->process_key_event = ibus_akaza_engine_process_key_event;
  engine_class->candidate_clicked = ibus_akaza_engine_candidate_clicked;
}


void ibus_akaza_set_callback(
    void* context,
    ibus_akaza_callback_key_event* cb,
    ibus_akaza_callback_candidate_clicked* candidate_cb
) {
    global_context = context;
    global_key_event_cb = cb;
    global_candidate_clicked_cb = candidate_cb;
}

void ibus_akaza_init(bool ibus) {
  printf("Akaza bootstrap(in wrapper.c)\n");

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

