import kclvm.config as config
import kclvm.api.version

from .main import Main

__all__ = ["Main"]

__version__ = kclvm.api.version.VERSION
config.version = __version__
