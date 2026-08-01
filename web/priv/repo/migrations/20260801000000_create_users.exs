defmodule Web.Repo.Migrations.CreateUsers do
  use Ecto.Migration

  def change do
    create table(:users) do
      add :email, :string, null: false
      add :name, :string
      add :google_uid, :string, null: false
      add :avatar, :string

      timestamps()
    end

    create unique_index(:users, [:google_uid])
    create unique_index(:users, [:email])
  end
end
