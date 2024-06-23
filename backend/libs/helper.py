import hashlib
import time
import re


def getmd5(code: str) -> str:
    """return md5 value of incoming code

    get md5 from code

    Args:
        code: str value

    Return:
        return md5 value of code
    """
    md5string = hashlib.md5(code.encode("utf-8"))
    return md5string.hexdigest()


def get_current_time() -> int:
    return int(time.time() * 1000)


def get_random_string(length: int) -> str:
    """return random string

    get random string with length

    Args:
        length: length of random string

    Return:
        return random string
    """
    import random
    import string

    return "".join(random.choices(string.ascii_letters + string.digits, k=length))


html_pattern = re.compile(r"<[^>]+>", re.S)


def contains_html_code(text: str) -> bool:
    match = html_pattern.search(text)
    if match:
        return True
    return False
