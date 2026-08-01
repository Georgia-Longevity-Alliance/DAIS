defmodule Web.Knowledge.Source do
  use Ecto.Schema
  import Ecto.Changeset

  @types ~w(journal_article preprint dataset experiment observation personal_communication software other)

  schema "sources" do
    field :title, :string
    field :source_type, :string, default: "other"
    field :url, :string
    field :doi, :string
    field :pmid, :string
    field :description, :string

    belongs_to :user, Web.Accounts.User

    timestamps()
  end

  def changeset(source, attrs) do
    source
    |> cast(attrs, [:title, :source_type, :url, :doi, :pmid, :description, :user_id])
    |> validate_required([:title, :user_id])
    |> validate_inclusion(:source_type, @types)
  end
end
