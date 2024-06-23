from flask.testing import FlaskClient

def test_request_example(client: FlaskClient):
    response = client.get("/health")
    json = response.json
    data = json.get("data")
    
    assert data["status"] == "ok"