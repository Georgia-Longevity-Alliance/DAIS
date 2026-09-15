defmodule Web.Repo.Migrations.AddNoepediaToClaims do
  use Ecto.Migration

  def change do
    alter table(:knowledge_claims) do
      # Noepedia: replication as a first-class operation (independent repetition)
      add :replication_count, :integer, default: 0
      # Noepedia: VALID-IN-CONTEXT — conditions under which the claim applies
      add :valid_in, :string
      add :valid_in_context, :map, default: %{}
      # Noepedia: coverage — relevant networks identified vs evaluated
      add :coverage_relevant, :integer, default: 0
      add :coverage_evaluated, :integer, default: 0
      add :coverage_not_yet, :integer, default: 0
      # Noepedia: OPEN/CONFLICT are legal knowledge states
      # (status already supports proposed|supported|tested|replicated|contested|refuted|superseded|open)
    end
  end
end
