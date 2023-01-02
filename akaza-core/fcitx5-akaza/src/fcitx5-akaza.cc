#include "fcitx5-akaza.h"

void AkazaEngine::keyEvent(const fcitx::InputMethodEntry& entry, fcitx::KeyEvent& keyEvent)
{
    FCITX_UNUSED(entry);
    FCITX_INFO() << keyEvent.key() << " isRelease=" << keyEvent.isRelease();
}

FCITX_ADDON_FACTORY(AkazaEngineFactory);

void hello() {
}
