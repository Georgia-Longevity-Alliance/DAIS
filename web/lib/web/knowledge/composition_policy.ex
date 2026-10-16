defmodule Web.Knowledge.CompositionPolicy do
  @moduledoc """
  Justification-composition semantics (DCT19–DCT20).

  > Multiple justifications ≠ redundancy. `same result ≠ same provenance`.

  Whether a result survives a challenge depends on an explicit composition
  policy: ALTERNATIVE (any one sufficient path) vs JOINT (all required
  together). If no policy applies, evaluation stops — it never fabricates.
  """
  use Ecto.Schema
  import Ecto.Changeset

  schema "noepedia_composition_policies" do
    field :result_id, :string
    field :composition, :string
    field :justification_ids, :map, default: %{}
    field :scope, :map, default: %{}

    timestamps()
  end

  def changeset(policy, attrs) do
    policy
    |> cast(attrs, [:result_id, :composition, :justification_ids, :scope])
    |> validate_required([:result_id, :composition])
    |> validate_inclusion(:composition, ~w(ALTERNATIVE JOINT))
  end
end
