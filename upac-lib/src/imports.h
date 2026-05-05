/* ── Workarounds for Zig translate-c ────────────────────────────────────────
 *
 * Zig 0.16's translate-c cannot parse top-level _Pragma(...) expansions, nor
 * __builtin_constant_p inside inline functions, nor __typeof__-based casts.
 *
 * Strategy: pre-include problematic sub-headers (spoofing __GLIB_H_INSIDE__ to
 * bypass their direct-include guard) so that their own include guards fire.
 * Then neutralize the offending macros. When the real headers are included the
 * guards prevent re-processing, so our overrides survive throughout.
 */

/* 1. gversion.h (__G_VERSION_H__): pre-include so GLIB_MAJOR_VERSION and
 *    GLIB_MINOR_VERSION are defined before gmacros.h pulls in gversionmacros.h
 *    (which needs them to compute GLIB_VERSION_CUR_STABLE). */
#define __GLIB_H_INSIDE__
#include <glib/gversion.h>
#undef __GLIB_H_INSIDE__

/* 2. gmacros.h (__G_MACROS_H__): kill _Pragma macros. */
#define __GLIB_H_INSIDE__
#include <glib/gmacros.h>
#undef __GLIB_H_INSIDE__

#undef G_GNUC_BEGIN_IGNORE_DEPRECATIONS
#define G_GNUC_BEGIN_IGNORE_DEPRECATIONS

#undef G_GNUC_END_IGNORE_DEPRECATIONS
#define G_GNUC_END_IGNORE_DEPRECATIONS

/* 3. glib-typeof.h (__GLIB_TYPEOF_H__): replace __typeof__ with void* so the
 *    g_object_ref macro produces a (void*) cast instead of a typeof-based cast
 *    that translate-c cannot emit with a known result type. */
#define __GLIB_H_INSIDE__
#include <glib/glib-typeof.h>
#undef __GLIB_H_INSIDE__

#undef glib_typeof
#define glib_typeof(t) void*

/* 4. gstring.h (__G_STRING_H__): kill g_string_free macro.
 *    glib-autocleanups.h calls g_string_free(str, TRUE) inside a static inline
 *    with TRUE as a compile-time constant, expanding __builtin_constant_p code
 *    that translate-c cannot render as valid Zig. */
#define __GLIB_H_INSIDE__
#include <glib/gstring.h>
#undef __GLIB_H_INSIDE__

#undef g_string_free

/* ── Real headers ──────────────────────────────────────────────────────── */
#include <glib.h>
#include <gio/gio.h>
#include <glib-unix.h>
#include <ostree.h>
#include <sys/statvfs.h>
