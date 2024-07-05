# -*- coding: utf-8 -*-

import logging
import sys
from typing import Dict, Optional

loggers: Dict[str, logging.Logger] = {}


def get_logger(name: Optional[str] = None) -> logging.Logger:
    """获得一个logger 实例，用来打印日志
    Args:
        name: logger的名称
    Return:
        返回一个logger实例
    """
    global loggers

    if not name:
        name = __name__

    has = loggers.get(name)
    if has:
        return has

    logger = logging.getLogger(name=name)
    logger.setLevel(logging.DEBUG)
    stream_handler = logging.StreamHandler(sys.stdout)
    logger.addHandler(stream_handler)

    loggers[name] = logger

    return logger
