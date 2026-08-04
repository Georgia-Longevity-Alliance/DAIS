defmodule Web.Interview do
  @moduledoc """
  Socratic passport interview — deterministic state machine.

  Mirrors py_backend/aisocket/passport_interview.py in Elixir.
  Each `next/2` call receives the user's response and returns
  the next question + updated state.
  """

  @type phase ::
          :greeting
          | :device_identity
          | :capabilities
          | :capability_detail
          | :forbidden
          | :forbidden_detail
          | :autonomous
          | :emergency
          | :review
          | :done

  @type t :: %__MODULE__{
          phase: phase(),
          device_name: String.t(),
          device_description: String.t(),
          device_platform: String.t(),
          capabilities: [map()],
          cap_idx: integer(),
          cap_detail_step: integer(),
          forbidden_actions: [map()],
          forbidden_idx: integer(),
          can_autonomous: boolean(),
          autonomous_max_hours: float(),
          autonomous_actions: [String.t()],
          on_anomaly: String.t(),
          on_power_loss: String.t(),
          risk_class: String.t(),
          emergency_contacts: [map()],
          question_count: integer(),
          messages: [map()]
        }

  defstruct phase: :greeting,
            device_name: "",
            device_description: "",
            device_platform: "",
            capabilities: [],
            cap_idx: -1,
            cap_detail_step: 0,
            forbidden_actions: [],
            forbidden_idx: -1,
            can_autonomous: false,
            autonomous_max_hours: 0.0,
            autonomous_actions: [],
            on_anomaly: "pause_and_notify",
            on_power_loss: "safe_shutdown",
            risk_class: "medium",
            emergency_contacts: [],
            question_count: 0,
            messages: [],
            auto_step: 0,
            emerg_step: 0,
            emerg_draft: %{}

  # ── Public API ──

  @doc "Start a new interview. Returns {message, state}."
  def start do
    state = %__MODULE__{
      messages: [
        %{role: "interviewer", text: greeting_message()}
      ]
    }
    {greeting_message(), state}
  end

  @doc """
  Process user response, return {next_question, updated_state}.
  The state.messages list accumulates the full conversation history.
  """
  def next(%__MODULE__{} = state, user_text) do
    state = %{
      state
      | question_count: state.question_count + 1,
        messages: state.messages ++ [%{role: "user", text: user_text}]
    }

    {new_state, reply} = handle_phase(state, user_text)

    new_state = %{
      new_state
      | messages: new_state.messages ++ [%{role: "interviewer", text: reply}]
    }

    {reply, new_state}
  end

  @doc "Build passport JSON from completed state."
  def build_passport(%__MODULE__{phase: :done} = state) do
    %{
      "device_id" => uuid_v4(),
      "name" => state.device_name,
      "description" => state.device_description,
      "capabilities" => Enum.map(state.capabilities, &build_capability/1),
      "forbidden_always" => Enum.map(state.forbidden_actions, &build_forbidden/1),
      "risk_class" => state.risk_class,
      "autonomous_mandate" => build_autonomous(state),
      "emergency_contacts" => state.emergency_contacts,
      "platform" => state.device_platform,
      "version" => %{"major" => 1, "minor" => 0, "patch" => 0},
      "signature" => nil
    }
  end

  def build_passport(_), do: {:error, :not_done}

  @doc "Is the interview complete?"
  def done?(%__MODULE__{phase: :done}), do: true
  def done?(_), do: false

  # ── Phase Handlers ──

  defp handle_phase(%{phase: :greeting} = state, _msg) do
    q = q(:device_identity, "name")
    {%{state | phase: :device_identity}, q}
  end

  defp handle_phase(%{phase: :device_identity} = state, msg) do
    cond do
      state.device_name == "" ->
        {%{state | device_name: String.trim(msg)}, q(:device_identity, "description")}

      state.device_description == "" ->
        {%{state | device_description: String.trim(msg)}, q(:device_identity, "platform")}

      true ->
        platform = normalize_platform(msg)
        {intro, q_name} = {capability_intro(), q(:capabilities, "name")}
        {%{state | device_platform: platform, phase: :capabilities}, intro <> "\n\n" <> q_name}
    end
  end

  # Capabilities: collecting names
  defp handle_phase(%{phase: :capabilities} = state, msg) do
    answer = String.trim(msg) |> String.downcase()

    if answer in ~w(done no нет finished всё) do
      if state.capabilities == [] do
        {state, "You haven't listed any capabilities yet. What can your device do?"}
      else
        q = forbidden_intro() <> "\n\n" <> q(:forbidden, "name")
        {%{state | phase: :forbidden, forbidden_idx: -1}, q}
      end
    else
      cap = %{
        "name" => snake_case(answer),
        "description" => "",
        "parameters" => [],
        "risk" => "low"
      }

      state = %{state | capabilities: state.capabilities ++ [cap], cap_idx: length(state.capabilities), cap_detail_step: 0, phase: :capability_detail}
      q = "Got it: **#{cap["name"]}**.\n\n#{q(:capability_detail, "description")}"
      {state, q}
    end
  end

  # Capability detail — 6 sub-steps (matches Python version)
  defp handle_phase(%{phase: :capability_detail} = state, msg) do
    idx = state.cap_idx
    cap = Enum.at(state.capabilities, idx)
    step = state.cap_detail_step
    msg_lower = String.trim(msg) |> String.downcase()

    case step do
      # Step 0: description just set → ask has_params
      0 ->
        cap = %{cap | "description" => String.trim(msg)}
        caps = List.replace_at(state.capabilities, idx, cap)
        {%{state | capabilities: caps, cap_detail_step: 1}, q(:capability_detail, "has_params")}

      # Step 1: answer to has_params → branch
      1 ->
        if msg_lower in ~w(no нет none) do
          {%{state | cap_detail_step: 4}, q(:capability_detail, "risk")}
        else
          {%{state | cap_detail_step: 2}, q(:capability_detail, "param_name")}
        end

      # Step 2: collecting param → ask more_params, move to step 3
      2 ->
        param = parse_param(String.trim(msg))
        params = cap["parameters"] ++ (param && [param] || [])
        cap = %{cap | "parameters" => params}
        caps = List.replace_at(state.capabilities, idx, cap)
        {%{state | capabilities: caps, cap_detail_step: 3}, q(:capability_detail, "more_params")}

      # Step 3: answer to "more_params?"
      3 ->
        if msg_lower in ~w(yes да y ещё) do
          {%{state | cap_detail_step: 2}, q(:capability_detail, "param_name")}
        else
          {%{state | cap_detail_step: 4}, q(:capability_detail, "risk")}
        end

      # Step 4: risk collected → ask more_capabilities
      4 ->
        risk = normalize_risk(msg)
        cap = %{cap | "risk" => risk}
        caps = List.replace_at(state.capabilities, idx, cap)
        {%{state | capabilities: caps, cap_detail_step: 5}, q(:capability_detail, "more_capabilities")}

      # Step 5: more_capabilities → transition
      5 ->
        if msg_lower in ~w(yes да y ещё) do
          {%{state | phase: :capabilities}, q(:capabilities, "name")}
        else
          q = forbidden_intro() <> "\n\n" <> q(:forbidden, "name")
          {%{state | phase: :forbidden, forbidden_idx: -1}, q}
        end
    end
  end

  # Forbidden: collecting names
  defp handle_phase(%{phase: :forbidden} = state, msg) do
    answer = String.trim(msg) |> String.downcase()

    cond do
      answer in ~w(done no нет finished всё) ->
        {%{state | phase: :autonomous}, q(:autonomous, "can_autonomous")}

      answer in ~w(yes да y ещё) ->
        {state, q(:forbidden, "name")}

      true ->
        fa = %{"name" => snake_case(answer), "reason" => "", "constitutional" => false}
        idx = length(state.forbidden_actions)
        state = %{state | forbidden_actions: state.forbidden_actions ++ [fa], forbidden_idx: idx, phase: :forbidden_detail}
        q = "**#{fa["name"]}** — forbidden.\n\n#{q(:forbidden_detail, "reason")}"
        {state, q}
    end
  end

  # Forbidden detail: reason + constitutional
  defp handle_phase(%{phase: :forbidden_detail} = state, msg) do
    idx = state.forbidden_idx
    fa = Enum.at(state.forbidden_actions, idx)

    if fa["reason"] == "" do
      fa = %{fa | "reason" => String.trim(msg)}
      fas = List.replace_at(state.forbidden_actions, idx, fa)
      {%{state | forbidden_actions: fas}, q(:forbidden_detail, "constitutional")}
    else
      const = String.trim(msg) |> String.downcase() |> then(&(&1 in ~w(yes да y absolute true)))
      fa = %{fa | "constitutional" => const}
      fas = List.replace_at(state.forbidden_actions, idx, fa)
      {%{state | forbidden_actions: fas, phase: :forbidden}, q(:forbidden, "more_forbidden")}
    end
  end

  # Autonomous
  defp handle_phase(%{phase: :autonomous} = state, msg) do
    msg_lower = String.trim(msg) |> String.downcase()

    cond do
      not state.can_autonomous ->
        can = msg_lower in ~w(yes да y true can может)
        if can do
          {%{state | can_autonomous: true}, q(:autonomous, "max_hours")}
        else
          {%{state | phase: :emergency}, q(:emergency, "risk_class")}
        end

      state.autonomous_max_hours == 0.0 ->
        hours = parse_float(msg, 1.0)
        {%{state | autonomous_max_hours: hours}, q(:autonomous, "actions")}

      state.autonomous_actions == [] ->
        actions =
          String.split(msg, ~r/[,;]\s*/, trim: true)
          |> Enum.map(&snake_case(String.trim(&1)))
        {%{state | autonomous_actions: actions}, q(:autonomous, "on_anomaly")}

      state.on_anomaly == "pause_and_notify" and msg_lower not in ~w(pause_and_notify safe_shutdown continue_with_limits invoke_llm пауза остановка) ->
        # This is a simplification — in the Python version we track step count.
        # Here we use the default value as sentinel.
        # Actually we need step tracking. Let's add auto_step.
        handle_auto_step(state, msg)

      true ->
        handle_auto_step(state, msg)
    end
  end

  # Emergency
  defp handle_phase(%{phase: :emergency} = state, msg) do
    step = state.emerg_step

    case step do
      0 ->
        risk = normalize_risk(msg)
        {%{state | risk_class: risk, emerg_step: 1}, q(:emergency, "has_emergency_contact")}

      1 ->
        answer = String.trim(msg) |> String.downcase()
        if answer in ~w(no нет none skip) do
          {%{state | phase: :review}, review_summary(state)}
        else
          {%{state | emerg_step: 2}, q(:emergency, "contact_name")}
        end

      2 ->
        contact = %{"name" => String.trim(msg)}
        {%{state | emerg_step: 3, emerg_draft: contact}, q(:emergency, "contact_method")}

      3 ->
        method = String.trim(msg) |> String.downcase()
        contact = Map.put(state.emerg_draft, "method_type", if(String.contains?(method, "phone"), do: "phone", else: "email"))
        {%{state | emerg_step: 4, emerg_draft: contact}, q(:emergency, "contact_value")}

      4 ->
        value = String.trim(msg)
        type = Map.get(state.emerg_draft, "method_type", "email")
        method_map = %{type => value}
        contact = Map.put(state.emerg_draft, "method", method_map)
        {%{state | emerg_step: 5, emerg_draft: contact}, q(:emergency, "contact_priority")}

      5 ->
        priority = parse_int(msg, 2)
        contact = Map.put(state.emerg_draft, "priority", priority)
        contacts = state.emergency_contacts ++ [contact]
        {%{state | emergency_contacts: contacts, emerg_step: 6, emerg_draft: %{}}, q(:emergency, "more_contacts")}

      6 ->
        answer = String.trim(msg) |> String.downcase()
        if answer in ~w(yes да y ещё) do
          {%{state | emerg_step: 2}, q(:emergency, "contact_name")}
        else
          {%{state | phase: :review}, review_summary(state)}
        end
    end
  end

  # Review
  defp handle_phase(%{phase: :review} = state, msg) do
    answer = String.trim(msg) |> String.downcase()

    if answer in ~w(yes да ok confirm подтверждаю y) do
      {%{state | phase: :done},
       "✅ **Passport created!**\n\n" <>
         "Device: **#{state.device_name}**\n" <>
         "Capabilities: #{length(state.capabilities)}\n" <>
         "Forbidden: #{length(state.forbidden_actions)}\n" <>
         "Risk: #{state.risk_class}\n\n" <>
         "The passport JSON is ready for registration."}
    else
      {state, "What would you like to change? (Or type 'yes' to confirm)"}
    end
  end

  defp handle_phase(%{phase: :done} = state, _msg) do
    {state, "This interview is complete. Your passport is ready."}
  end

  # ── Autonomous helpers ──

  defp handle_auto_step(state, msg) do
    _msg_lower = String.trim(msg) |> String.downcase()
    step = state.auto_step

    case step do
      0 ->
        {%{state | on_anomaly: normalize_anomaly(msg), auto_step: 1}, q(:autonomous, "on_power_loss")}

      1 ->
        {%{state | on_power_loss: normalize_power_loss(msg), auto_step: 2, phase: :emergency}, q(:emergency, "risk_class")}
    end
  end

  # ── Messages ──

  defp greeting_message do
    """
    👋 **Welcome to the DAISocket Passport Interview!**

    I'll help you create a **Passport** for your device — a document that tells any AI what your device can do, what it must NEVER do, and how to safely control it.

    Ready? Let's start!
    """
  end

  defp capability_intro do
    """
    Now let's list what your device **CAN do**.

    Think of every action — motors, lasers, sensors, heaters, cameras...

    I'll ask for one capability at a time, then details about each.
    """
  end

  defp forbidden_intro do
    """
    Now the most critical part: what must your device **NEVER** do?

    These are enforced in firmware — no AI and no emergency can override them.
    """
  end

  defp review_summary(state) do
    caps =
      state.capabilities
      |> Enum.map(&"  - #{&1["name"]}: #{String.slice(&1["description"], 0, 60)}...")
      |> Enum.join("\n")

    forbids =
      state.forbidden_actions
      |> Enum.map(&"  - #{&1["name"]} (constitutional=#{&1["constitutional"]})")
      |> Enum.join("\n")

    contacts =
      state.emergency_contacts
      |> Enum.map(&"  - [#{&1["priority"]}] #{&1["name"]}")
      |> Enum.join("\n")

    """
    📋 **PASSPORT REVIEW**

    **Device:** #{state.device_name}
    **Platform:** #{state.device_platform}
    **Risk class:** #{state.risk_class}

    **Capabilities (#{length(state.capabilities)}):**
    #{if caps == "", do: "  (none)", else: caps}

    **Forbidden (#{length(state.forbidden_actions)}):**
    #{if forbids == "", do: "  (none)", else: forbids}

    **Autonomous:** #{if state.can_autonomous, do: "Yes (#{state.autonomous_max_hours}h)", else: "No"}
    **Emergency contacts:**
    #{if contacts == "", do: "  (none)", else: contacts}

    ---
    Type **yes** to confirm, or tell me what to change.
    """
  end

  # ── Question templates ──

  defp q(:device_identity, "name"),
    do: "What is your device called? Give it a short, memorable name (e.g., 'argus_os1_v6')."

  defp q(:device_identity, "description"),
    do: "Describe the device in 2-3 sentences. What does it do?"

  defp q(:device_identity, "platform"),
    do: "What hardware platform? (esp32, arduino, raspberry_pi, jetson, linux, android, ros2, browser)"

  defp q(:capabilities, "name"),
    do: "Name a capability (snake_case, e.g., 'move_stage_x', 'fire_laser', 'set_temperature')."

  defp q(:capability_detail, "description"),
    do: "What does this capability do? Describe in one sentence."

  defp q(:capability_detail, "has_params"),
    do: "Does this capability need parameters? (e.g., temperature value, speed, target position)"

  defp q(:capability_detail, "param_name"),
    do: "Parameter name? Format: 'name: type, required, min-max' (e.g., 'temperature: float, required, 20-39')"

  defp q(:capability_detail, "more_params"),
    do: "Any more parameters? (name, or 'no')"

  defp q(:capability_detail, "risk"),
    do: "Risk level?\n- informational: just reads data\n- low: minor effect (LED)\n- medium: motor, heater\n- high: laser, blade, potential harm\n- critical: life-critical (FORBIDDEN by DAIS)"

  defp q(:capability_detail, "more_capabilities"),
    do: "Any other capabilities? (yes/no)"

  defp q(:forbidden, "name"),
    do: "Name a forbidden action (snake_case, e.g., 'exceed_max_temp'). Or 'done'."

  defp q(:forbidden, "more_forbidden"),
    do: "Any other forbidden actions? (yes/no/done)"

  defp q(:forbidden_detail, "reason"),
    do: "WHY is this forbidden? What happens if violated?"

  defp q(:forbidden_detail, "constitutional"),
    do: "Is this ABSOLUTE — can NEVER be overridden, even in emergency? (yes = constitutional, no = can be overridden)"

  defp q(:autonomous, "can_autonomous"),
    do: "Can this device operate WITHOUT human supervision? (yes/no)"

  defp q(:autonomous, "max_hours"),
    do: "Maximum autonomous duration (hours)?"

  defp q(:autonomous, "actions"),
    do: "Which capabilities are allowed during autonomous operation? (comma-separated list)"

  defp q(:autonomous, "on_anomaly"),
    do: "What to do on anomaly?\n- pause_and_notify\n- safe_shutdown\n- continue_with_limits\n- invoke_llm"

  defp q(:autonomous, "on_power_loss"),
    do: "What on power loss?\n- safe_shutdown\n- save_state_and_sleep\n- switch_to_battery"

  defp q(:emergency, "risk_class"),
    do: "Overall risk class?\n- informational: display, speaker\n- low: LED, small motor\n- medium: heater, motor >1W\n- high: laser, heavy machinery\n- critical: ventilator, brake (FORBIDDEN)"

  defp q(:emergency, "has_emergency_contact"),
    do: "Add an emergency contact? (yes/no/skip)"

  defp q(:emergency, "contact_name"),
    do: "Contact name?"

  defp q(:emergency, "contact_method"),
    do: "Email or phone?"

  defp q(:emergency, "contact_value"),
    do: "Email address or phone number?"

  defp q(:emergency, "contact_priority"),
    do: "Priority? (1 = first to contact, 2 = backup)"

  defp q(:emergency, "more_contacts"),
    do: "Any other emergency contacts? (yes/no)"

  # ── Parsers + normalizers ──

  defp snake_case(text) do
    text
    |> String.trim()
    |> String.downcase()
    |> String.replace(~r/[^a-z0-9\s]/, " ")
    |> String.replace(~r/\s+/, "_")
    |> String.trim("_")
  end

  defp parse_param(text) do
    # Format: "name: type, required, min-max" or just "name"
    parts = String.split(text, ":", parts: 2)
    name = snake_case(hd(parts))

    param = %{"name" => name, "param_type" => "float", "required" => false}

    if length(parts) > 1 do
      details = Enum.at(parts, 1) |> String.trim() |> String.downcase()

      param = %{
        param
        | "param_type" => cond do
            String.contains?(details, "integer") -> "integer"
            String.contains?(details, "boolean") -> "boolean"
            String.contains?(details, "string") -> "string"
            true -> param["param_type"]
          end,
          "required" => String.contains?(details, "required")
      }

      # Parse min-max
      range_match = Regex.run(~r/(\d+\.?\d*)\s*-\s*(\d+\.?\d*)/, details)
      if range_match do
        [_, min_s, max_s] = range_match
        constraints =
          (Map.get(param, "constraints") || %{})
          |> Map.put("min", parse_float(min_s, 0))
          |> Map.put("max", parse_float(max_s, 100))
        Map.put(param, "constraints", constraints)
      else
        param
      end
    else
      param
    end
  end

  defp normalize_platform(text) do
    t = String.trim(text) |> String.downcase()
    cond do
      String.contains?(t, "esp32") -> "esp32"
      String.contains?(t, "arduino") -> "arduino"
      String.contains?(t, "raspberry") -> "raspberry_pi"
      String.contains?(t, "jetson") -> "jetson"
      String.contains?(t, "android") -> "android"
      String.contains?(t, "ros") -> "ros2"
      String.contains?(t, "browser") -> "browser"
      String.contains?(t, "linux") -> "linux"
      true -> t
    end
  end

  defp normalize_risk(text) do
    t = String.trim(text) |> String.downcase()
    cond do
      String.contains?(t, "critical") -> "critical"
      String.contains?(t, "high") -> "high"
      String.contains?(t, "medium") -> "medium"
      String.contains?(t, "low") -> "low"
      String.contains?(t, "info") -> "informational"
      true -> "medium"
    end
  end

  defp normalize_anomaly(text) do
    t = String.trim(text) |> String.downcase()
    cond do
      String.contains?(t, "pause") or String.contains?(t, "stop") -> "pause_and_notify"
      String.contains?(t, "shutdown") or String.contains?(t, "emergency") -> "safe_shutdown"
      String.contains?(t, "continue") or String.contains?(t, "limit") -> "continue_with_limits"
      String.contains?(t, "llm") or String.contains?(t, "invoke") -> "invoke_llm"
      true -> "pause_and_notify"
    end
  end

  defp normalize_power_loss(text) do
    t = String.trim(text) |> String.downcase()
    cond do
      String.contains?(t, "save") or String.contains?(t, "sleep") -> "save_state_and_sleep"
      String.contains?(t, "battery") or String.contains?(t, "switch") -> "switch_to_battery"
      true -> "safe_shutdown"
    end
  end

  defp parse_float(text, default) do
    case Regex.run(~r/[\d.]+/, text) do
      [n] -> case Float.parse(n) do
        {val, _} -> val
        :error -> default
      end
      nil -> default
    end
  end

  defp parse_int(text, default) do
    case Regex.run(~r/\d+/, text) do
      [n] -> case Integer.parse(n) do
        {val, _} -> val
        :error -> default
      end
      nil -> default
    end
  end

  defp uuid_v4 do
    <<u0::48, _::4, u1::12, _::2, u2::62>> = :crypto.strong_rand_bytes(16)
    <<u0::48, 4::4, u1::12, 2::2, u2::62>>
    |> Base.encode16(case: :lower)
    |> then(fn s ->
      "#{String.slice(s, 0, 8)}-#{String.slice(s, 8, 4)}-#{String.slice(s, 12, 4)}-#{String.slice(s, 16, 4)}-#{String.slice(s, 20, 12)}"
    end)
  end

  # ── Capability & Forbidden builders ──

  defp build_capability(cap) do
    %{
      "name" => cap["name"],
      "description" => cap["description"],
      "parameters" => Enum.map(cap["parameters"], fn p ->
        param = %{
          "name" => p["name"],
          "param_type" => p["param_type"],
          "required" => p["required"]
        }
        if Map.has_key?(p, "constraints") do
          Map.put(param, "constraints", p["constraints"])
        else
          param
        end
      end),
      "risk" => cap["risk"]
    }
  end

  defp build_forbidden(fa) do
    %{
      "name" => fa["name"],
      "reason" => fa["reason"],
      "constitutional" => fa["constitutional"]
    }
  end

  defp build_autonomous(state) do
    if state.can_autonomous do
      %{
        "max_duration_seconds" => trunc(state.autonomous_max_hours * 3600),
        "allowed_offline_actions" => state.autonomous_actions,
        "on_anomaly" => state.on_anomaly,
        "on_power_loss" => state.on_power_loss
      }
    else
      nil
    end
  end
end
