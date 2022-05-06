import locale as _locale

import kclvm.kcl.error as kcl_error

LOCALE_LIST = list(_locale.locale_alias.keys())
INVALID_I18N_LOCALE_MSG = "invalid i18n locale, expected {}"


def check_locale(locale: str):
    """Check a locale string is a valid locale

    Parameters
    ----------
    locale: locale string
    """
    if (
        not locale
        or not isinstance(locale, str)
        or locale not in LOCALE_LIST
        or locale.replace("-", "_") not in LOCALE_LIST
    ):
        kcl_error.report_exception(
            err_type=kcl_error.ErrType.CompileError_TYPE,
            arg_msg=INVALID_I18N_LOCALE_MSG.format(", ".join(LOCALE_LIST)),
        )
