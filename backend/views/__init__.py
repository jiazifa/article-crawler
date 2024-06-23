from flask import Flask


def init_app(app: Flask) -> None:
    from extensions.ext_rest import rest_api

    from views import welcome

    rest_api.add_namespace(welcome.ns)

    return None
