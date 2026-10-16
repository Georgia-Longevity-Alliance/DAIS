defmodule Web.Knowledge.DependencyPolicy do
  @moduledoc """
  An addressable dependency policy (DCT05–DCT07): how an upstream status
  transition propagates downstream. Lives in the network with ROLE = POLICY,
  not in the demon core. A missing applicable policy is a lawful BLOCK
  (NEED / MISSING_RULE) — the downstream consequence is never invented.
  """
  use Ecto.Schema
  import Ecto.Changeset

  schema "noepedia_dependency_policies" do
    field :name, :string
    field :upstream_transition, :string
    field :downstream_consequence, :string
    field :rule_ref, :string
    field :role, :string, default: "POLICY"
    field :scope, :map, default: %{}

    timestamps()
  end

  def changeset(policy, attrs) do
    policy
    |> cast(attrs, [:name, :upstream_transition, :downstream_consequence,
                    :rule_ref, :role, :scope])
    |> validate_required([:name, :upstream_transition, :downstream_consequence])
  end
end
