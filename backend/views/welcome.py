from flask import current_app
from flask_restx import Namespace, Resource, fields, marshal_with
from extensions.ext_rest import rest_api
from libs.view_utils import ResponseModel, response_schema

ns = Namespace("", description="Health Check API", api=rest_api)
    
class HealthView(Resource):
    
    # define the API response model
    health_model = ns.model("HealthModel", {
        "status": fields.String(readonly=True, required=False, description="Health status"),
        "version": fields.String(readonly=True, required=False, description="API version")
    })
    
    out_schema = ns.clone(
        "HealthResponseModel",
        response_schema,
        {
            "data": fields.Nested(health_model),
        },
    )
    
    
    @marshal_with(out_schema)
    def get(self):
        resp = ResponseModel[dict](data={"status": "ok", "version": current_app.config["CURRENT_VERSION"] or "0.0.1"})
        return resp
    
ns.add_resource(HealthView, "/health")