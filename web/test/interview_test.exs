defmodule Web.InterviewTest do
  use ExUnit.Case
  alias Web.Interview

  test "full interview flow produces valid passport" do
    {_greeting, s} = Interview.start()
    assert s.phase == :greeting

    {_, s} = Interview.next(s, "skip")
    assert s.phase == :device_identity
    {_, s} = Interview.next(s, "argus_test")
    assert s.device_name == "argus_test"
    {_, s} = Interview.next(s, "A test microscope with laser")
    {_, s} = Interview.next(s, "jetson")
    assert s.phase == :capabilities

    {_, s} = Interview.next(s, "move_stage_x")
    assert s.phase == :capability_detail
    {_, s} = Interview.next(s, "Move stage along X")
    {_, s} = Interview.next(s, "yes")
    {_, s} = Interview.next(s, "position_um: float, required, 0-25000")
    {_, s} = Interview.next(s, "no")
    {_, s} = Interview.next(s, "low")
    {_, s} = Interview.next(s, "done")
    assert s.phase == :forbidden

    {_, s} = Interview.next(s, "exceed_power")
    assert s.phase == :forbidden_detail
    {_, s} = Interview.next(s, "Too much power is dangerous")
    {_, s} = Interview.next(s, "yes")
    assert s.phase == :forbidden
    {_, s} = Interview.next(s, "done")
    assert s.phase == :autonomous

    {_, s} = Interview.next(s, "yes")
    {_, s} = Interview.next(s, "12")
    {_, s} = Interview.next(s, "move_stage_x, emergency_stop")
    {_, s} = Interview.next(s, "pause_and_notify")
    {_, s} = Interview.next(s, "safe_shutdown")
    assert s.phase == :emergency

    {_, s} = Interview.next(s, "high")
    {_, s} = Interview.next(s, "yes")
    {_, s} = Interview.next(s, "Jaba")
    {_, s} = Interview.next(s, "email")
    {_, s} = Interview.next(s, "jaba@longevity.ge")
    {_, s} = Interview.next(s, "1")
    {_, s} = Interview.next(s, "no")
    assert s.phase == :review

    {_, s} = Interview.next(s, "yes")
    assert s.phase == :done
    assert Interview.done?(s)

    passport = Interview.build_passport(s)
    assert passport["name"] == "argus_test"
    assert length(passport["capabilities"]) == 1
    assert length(passport["forbidden_always"]) == 1
    assert passport["risk_class"] == "high"
    assert length(passport["emergency_contacts"]) == 1
    assert passport["autonomous_mandate"]["max_duration_seconds"] == 43200
  end

  test "simple device with no params" do
    {_, s} = Interview.start()
    {_, s} = Interview.next(s, "skip")
    {_, s} = Interview.next(s, "led_light")
    {_, s} = Interview.next(s, "A simple LED light")
    {_, s} = Interview.next(s, "esp32")
    {_, s} = Interview.next(s, "turn_on")
    {_, s} = Interview.next(s, "Turn on the light")
    {_, s} = Interview.next(s, "no")
    {_, s} = Interview.next(s, "low")
    {_, s} = Interview.next(s, "done")
    {_, s} = Interview.next(s, "done")
    {_, s} = Interview.next(s, "no")
    {_, s} = Interview.next(s, "low")
    {_, s} = Interview.next(s, "no")
    {_, s} = Interview.next(s, "yes")
    assert s.phase == :done

    passport = Interview.build_passport(s)
    assert passport["name"] == "led_light"
    assert length(passport["capabilities"]) == 1
    assert hd(passport["capabilities"])["parameters"] == []
  end

  test "snake_case device name" do
    {_, s} = Interview.start()
    {_, s} = Interview.next(s, "skip")
    {_, s} = Interview.next(s, "my_cool_device")
    assert s.device_name == "my_cool_device"
  end
end
