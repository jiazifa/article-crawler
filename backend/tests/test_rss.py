from sqlalchemy.orm import scoped_session
from services.rss.controller import RssController
from services.rss.schema import QuerySubscriptionOption
from libs.schema import PageRequest


def test_request_example(db_session: scoped_session):
    controller = RssController(db_session)
    url = "https://sspai.com/feed"
    sub_1 = controller.insert_subscription_from_url(url)
    is_subs_exsits = controller.is_subscription_exist(url)
    assert is_subs_exsits
    # query
    query_1_op = QuerySubscriptionOption(
        page=PageRequest.default(), ids=[sub_1.subscription_id]
    )
    query_1_result = controller.query_subscription(query_1_op)
    assert len(query_1_result.data) == 1
    # get first db
    query_db = query_1_result.data[0]
    links_count = len(query_db.links)

    # insert one more time
    result_2 = controller.insert_subscription_from_url(url)
    is_subs_exsits_2 = controller.is_subscription_exist(url)

    assert is_subs_exsits_2
    assert result_2.subscription_id == sub_1.subscription_id
    # query 2
    query_2_op = QuerySubscriptionOption(
        page=PageRequest.default(), ids=[result_2.subscription_id]
    )
    query_2_result = controller.query_subscription(query_2_op)
    assert len(query_2_result.data) == 1

    # get first db
    query_sub_2 = query_2_result.data[0]
    assert len(query_sub_2.links) == links_count
