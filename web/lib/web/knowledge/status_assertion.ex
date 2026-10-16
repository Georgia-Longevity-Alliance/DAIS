defmodule Web.Knowledge.StatusAssertion do
  @moduledoc """
  One addressable status assertion (non-destructive MARK, DCT11–DCT12).

  A status transition creates a NEW assertion (with an incrementing epoch) and
  keeps the prior assertions as historical trace; the current epistemic status
  is the last applicable assertion. `history_status` tracks the committed fact
  (COMMITTED / ARCHIVED / COMPACTED / SUPERSEDED) independently.
  """
  use Ecto.Schema
  import Ecto.Changeset

  schema "noepedia_status_assertions" do
    field :record_id, :string
    field :status, :string
    field :epoch, :integer, default: 1
    field :scope, :map, default: %{}
    field :asserted_by, :string
    field :history_status, :string, default: "COMMITTED"

    timestamps()
  end

  def changeset(assertion, attrs) do
    assertion
    |> cast(attrs, [:record_id, :status, :epoch, :scope, :asserted_by, :history_status])
    |> validate_required([:record_id, :status])
    |> update_change(:status, &String.upcase/1)
    |> validate_inclusion(:status, ~w(PROPOSED SUPPORTED TESTED REPLICATED CONTESTED REFUTED SUPERSEDED OPEN VALID_IN_CONTEXT STALE))
  end
end
