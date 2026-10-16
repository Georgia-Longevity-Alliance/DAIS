defmodule Web.Repo.Migrations.AddDemonCoreTables do
  use Ecto.Migration

  def change do
    # Non-destructive status assertions (Demon Core Harvest, DCT11–DCT12):
    # every status transition is a NEW addressable assertion with an epoch;
    # the prior assertions remain as historical trace (never an overwrite).
    create table(:noepedia_status_assertions) do
      add :record_id, :binary_id
      add :status, :string
      add :epoch, :integer, default: 1
      add :scope, :map, default: %{}
      add :asserted_by, :binary_id
      add :history_status, :string, default: "COMMITTED"

      timestamps()
    end

    create index(:noepedia_status_assertions, [:record_id])

    # STOP CARDS (DEMON_CORE_SPEC §8): a stuck branch returns this instead of
    # fabricating a continuation. Stopping at the right place is success.
    create table(:noepedia_stop_cards) do
      add :session_id, :binary_id
      add :where_str, :string
      add :wanted, :text
      add :available, :text
      add :missing, :text
      add :stop_class, :string
      add :can_ask_outside, :boolean, default: false
      add :query, :text
      add :scope, :map, default: %{}

      timestamps()
    end

    # Dependency policies live in the network (ROLE = POLICY), not the demon
    # core (DCT05–DCT07). Missing applicable policy → NEED / MISSING_RULE.
    create table(:noepedia_dependency_policies) do
      add :name, :string
      add :upstream_transition, :string
      add :downstream_consequence, :string
      add :rule_ref, :binary_id
      add :role, :string, default: "POLICY"
      add :scope, :map, default: %{}

      timestamps()
    end

    # Justification-composition semantics (DCT19–DCT20): multiple
    # justifications ≠ redundancy; ALTERNATIVE vs JOINT is explicit.
    create table(:noepedia_composition_policies) do
      add :result_id, :binary_id
      add :composition, :string
      add :justification_ids, :map, default: %{}
      add :scope, :map, default: %{}

      timestamps()
    end
  end
end
