#!/bin/bash
bindgen --opaque-type IBusLookupTable --opaque-type IBusEngine --opaque-type IBusText --opaque-type IBusBus wrapper.h -- $(pkg-config ibus-1.0 --cflags)
