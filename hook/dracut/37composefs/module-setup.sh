#!/usr/bin/bash
# SPDX-FileCopyrightText: The composefs-rs contributors
# SPDX-FileCopyrightText: 2026 JustPav
# SPDX-FileCopyrightText: 2026 SmoothTeam
#
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Adapted from containers/composefs-rs's examples/*/extra/usr/lib/dracut/modules.d/37composefs/
# (MIT OR Apache-2.0). Only change from upstream: dropped the examples' debug-only strace inst.

check() {
    return 0
}

depends() {
    return 0
}

install() {
    inst \
        "${moddir}/composefs-setup-root" /bin/composefs-setup-root
    inst \
        "${moddir}/composefs-setup-root.service" \
        "${systemdsystemunitdir}/composefs-setup-root.service"

    $SYSTEMCTL -q --root "${initdir}" add-wants \
        'initrd-root-fs.target' 'composefs-setup-root.service'
}
