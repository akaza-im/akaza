#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# ibus-akaza: ibus engine for japanese characters
#
# Copyright (c) 2020 Tokuhiro Matsuno <tokuhirom@gmail.com>
#
# based on https://github.com/ibus/ibus-tmpl/
#
# This program is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; either version 3, or (at your option)
# any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program; if not, write to the Free Software
# Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.

import gi

gi.require_version('IBus', '1.0')
from gi.repository import IBus
from gi.repository import GLib
from gi.repository import GObject

import os
import sys
import getopt
import locale
import logging

# set_prgname before importing factory to show the name in warning
# messages when import modules are failed. E.g. Gtk.
GLib.set_prgname('ibus-engine-akaza')

logging.basicConfig(level=logging.DEBUG, filename='/tmp/ibus-akaza.log', filemode='w')
logging.info("Loading ibus-akaza")

__base_dir__ = os.path.dirname(__file__)


# TODO: ユーザー辞書の保存をbackground thread で実施するようにする

###########################################################################
# the app (main interface to ibus)

class IMApp:
    def __init__(self, exec_by_ibus):
        if not exec_by_ibus:
            global debug_on
            debug_on = True

        logging.info("Loading IMApp")

        from ibus_akaza.ui import AkazaIBusEngine

        self.mainloop = GLib.MainLoop()
        self.bus = IBus.Bus()
        self.bus.connect("disconnected", self.bus_disconnected_cb)
        self.factory = IBus.Factory.new(self.bus.get_connection())
        self.factory.add_engine("akaza", GObject.type_from_name("AkazaIBusEngine"))

        if exec_by_ibus:
            self.bus.request_name("org.freedesktop.IBus.Akaza", 0)
        else:
            xml_path = os.path.join(__base_dir__, 'AkazaIBusEngine.xml')
            if os.path.exists(xml_path):
                component = IBus.Component.new_from_file(xml_path)
            else:
                xml_path = os.path.join(os.path.dirname(__base_dir__),
                                        'ibus', 'component', 'akaza.xml')
                component = IBus.Component.new_from_file(xml_path)
            self.bus.register_component(component)

    def run(self):
        self.mainloop.run()

    def bus_disconnected_cb(self, bus):
        self.mainloop.quit()


def launch_engine(exec_by_ibus):
    IBus.init()
    IMApp(exec_by_ibus).run()


def print_help(out):
    print("-i, --ibus             executed by IBus.", file=out)
    print("-h, --help             show this message.", file=out)
    print("-d, --daemonize        daemonize ibus", file=out)


def main():
    try:
        locale.setlocale(locale.LC_ALL, "")
    except:
        pass

    exec_by_ibus = False
    daemonize = False

    shortopt = "ihd"
    longopt = ["ibus", "help", "daemonize"]

    try:
        opts, args = getopt.getopt(sys.argv[1:], shortopt, longopt)
    except getopt.GetoptError:
        print_help(sys.stderr)
        sys.exit(1)

    for o, a in opts:
        if o in ("-h", "--help"):
            print_help(sys.stdout)
            sys.exit(0)
        elif o in ("-d", "--daemonize"):
            daemonize = True
        elif o in ("-i", "--ibus"):
            exec_by_ibus = True
        else:
            print("Unknown argument: %s" % o, file=sys.stderr)
            print_help(sys.stderr)
            sys.exit(1)

    if daemonize:
        if os.fork():
            sys.exit()

    launch_engine(exec_by_ibus)


if __name__ == "__main__":
    try:
        main()
    except:
        logging.error("Cannot initialize", exc_info=True)
        sys.exit(1)
