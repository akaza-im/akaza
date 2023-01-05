#!/bin/bash
bindgen --opaque-type IBusSerializable --opaque-type IBusAttribute --opaque-type IBusAttrList --opaque-type IBusEngine --opaque-type IBusBus /usr/include/ibus-1.0/ibus.h -- $(pkg-config ibus-1.0 --cflags)
