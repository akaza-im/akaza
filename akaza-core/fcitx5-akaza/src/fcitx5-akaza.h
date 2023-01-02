#ifndef _FCITX5_AKAZA_AKAZA_H_
#define _FCITX5_AKAZA_AKAZA_H_

#include <fcitx/inputmethodengine.h>
#include <fcitx/addonfactory.h>

class AkazaEngine : public fcitx::InputMethodEngineV2 {
    void keyEvent(const fcitx::InputMethodEntry & entry, fcitx::KeyEvent & keyEvent) override;
};

class AkazaEngineFactory : public fcitx::AddonFactory {
    fcitx::AddonInstance * create(fcitx::AddonManager * manager) override {
        FCITX_UNUSED(manager);
        return new AkazaEngine;
    }
};

#endif // _FCITX5_AKAZA_AKAZA_H_
