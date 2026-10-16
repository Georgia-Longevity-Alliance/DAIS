defmodule WebWeb.DemonCoreApiTest do
  use WebWeb.ConnCase

  alias Web.Repo
  alias Web.Knowledge.StatusAssertion

  @moduledoc """
  End-to-end tests for the Demon Core API (Demon Core Harvest 2026-10-16,
  DCT01–DCT20): non-destructive status assertions, STOP CARDS, dependency
  policies, composition policies.
  """

  test "status assertions are non-destructive (epoch increments, history kept)" do
    Repo.delete_all(StatusAssertion)

    conn =
      post(build_conn(), ~p"/api/knowledge/status-assertions", %{
        "record_id" => "R1",
        "status" => "PROPOSED",
        "asserted_by" => "A1"
      })
    assert json_response(conn, 201)["epoch"] == 1

    conn =
      post(build_conn(), ~p"/api/knowledge/status-assertions", %{
        "record_id" => "R1",
        "status" => "SUPPORTED",
        "asserted_by" => "A1"
      })
    assert json_response(conn, 201)["epoch"] == 2

    conn = get(build_conn(), ~p"/api/knowledge/records/R1/status-assertions")
    data = json_response(conn, 200)
    assert data["count"] == 2
    assert ["PROPOSED", "SUPPORTED"] =
             Enum.map(data["assertions"], & &1["status"])
  end

  test "status assertion rejects an invalid status" do
    conn =
      post(build_conn(), ~p"/api/knowledge/status-assertions", %{
        "record_id" => "R2",
        "status" => "FACT"
      })
    assert json_response(conn, 422)
  end

  test "stop card records a NEED with classification (ASK=false)" do
    conn =
      post(build_conn(), ~p"/api/knowledge/stop-cards", %{
        "session_id" => "S1",
        "where" => "DCT20 / one convergent justification challenged",
        "wanted" => "determine survival",
        "missing" => "justification-composition policy",
        "stop_class" => "MISSING_RULE",
        "can_ask_outside" => false
      })
    body = json_response(conn, 201)
    assert body["stop_class"] == "MISSING_RULE"
    assert body["can_ask_outside"] == false
  end

  test "dependency policy is created with ROLE POLICY" do
    conn =
      post(build_conn(), ~p"/api/knowledge/dependency-policies", %{
        "name" => "premise_conflict_propagates",
        "upstream_transition" => "PREMISE -> CONTESTED",
        "downstream_consequence" => "CONTESTED",
        "rule_ref" => "E1"
      })
    body = json_response(conn, 201)
    assert body["role"] == "POLICY"
    assert body["downstream_consequence"] == "CONTESTED"
  end

  test "composition policy JOINT stores justification ids" do
    conn =
      post(build_conn(), ~p"/api/knowledge/composition-policies", %{
        "result_id" => "EV1",
        "composition" => "JOINT",
        "justification_ids" => ["J1", "J2"]
      })
    body = json_response(conn, 201)
    assert body["composition"] == "JOINT"
    assert body["justification_ids"]["ids"] == ["J1", "J2"]
  end
end
