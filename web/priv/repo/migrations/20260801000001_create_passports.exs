defmodule Web.Repo.Migrations.CreatePassports do
  use Ecto.Migration

  def change do
    create table(:passports) do
      add :device_id, :string, null: false
      add :name, :string
      add :description, :text
      add :platform, :string
      add :risk_class, :string
      add :payload, :map
      add :user_id, :string
      add :user_email, :string

      timestamps()
    end

    create index(:passports, [:device_id])
    create index(:passports, [:user_id])
  end
end
