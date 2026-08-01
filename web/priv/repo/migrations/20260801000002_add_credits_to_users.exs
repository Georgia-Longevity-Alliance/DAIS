defmodule Web.Repo.Migrations.AddCreditsToUsers do
  use Ecto.Migration

  def change do
    alter table(:users) do
      add :credits_balance, :float, default: 0.0
      add :credits_earned_total, :float, default: 0.0
      add :credits_spent_total, :float, default: 0.0
      add :role, :string, default: "contributor"
      add :reputation_score, :float, default: 0.0
      add :contributions_count, :integer, default: 0
    end
  end
end
