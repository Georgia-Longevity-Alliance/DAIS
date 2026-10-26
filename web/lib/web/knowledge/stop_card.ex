defmodule Web.Knowledge.StopCard do
  @moduledoc """
  A STOP CARD (AUDITOR_CORE_SPEC §8): a stuck branch returns this instead of
  fabricating a plausible continuation. Stopping at the right place is a
  successful behavior, not a failure. `can_ask_outside` distinguishes ASK
  (precise permitted query exists) from NEED (structure missing).
  """
  use Ecto.Schema
  import Ecto.Changeset

  @need_classes ~w(MISSING_FACT MISSING_RULE MISSING_ADDRESS MISSING_RELATION
                   MISSING_PATTERN MISSING_SCOPE MISSING_PROVENANCE MISSING_OPERATION
                   AMBIGUOUS_MATCH CONFLICT CLOSURE_UNKNOWN CREDIT_UNRESOLVED)

  schema "aletheia_stop_cards" do
    field :session_id, :string
    field :where_str, :string
    field :wanted, :string
    field :available, :string
    field :missing, :string
    field :stop_class, :string
    field :can_ask_outside, :boolean, default: false
    field :query, :string
    field :scope, :map, default: %{}

    timestamps()
  end

  def changeset(card, attrs) do
    card
    |> cast(attrs, [:session_id, :where_str, :wanted, :available, :missing,
                    :stop_class, :can_ask_outside, :query, :scope])
    |> validate_required([:where_str, :stop_class])
    |> validate_inclusion(:stop_class, @need_classes)
  end
end
