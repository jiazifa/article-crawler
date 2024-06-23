from flask import Flask
from flask_restx import Api
from libs.view_utils import response_context_model, response_schema

rest_api = Api(title="api", version="1.0", description="api", doc="/doc")

def init_app(app: Flask) -> None:
    rest_api.init_app(app)

    rest_api.models["ResponseContextModel"] = response_context_model
    rest_api.models["ResponseModel"] = response_schema
    
    return None
    