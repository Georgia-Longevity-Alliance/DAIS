defmodule Web.Knowledge.Claim do
  use Ecto.Schema
  import Ecto.Changeset

  @statuses ~w(proposed supported tested replicated contested refuted superseded open)

  schema "knowledge_claims" do
    field :subject, :string
    field :predicate, :string
    field :object, :string
    field :status, :string, default: "proposed"
    field :confidence, :float, default: 0.5
    field :domain, :string
    field :citation_count, :integer, default: 0
    field :reward_paid, :float, default: 0.0
    # Noepedia: replication as a first-class operation
    field :replication_count, :integer, default: 0
    # Noepedia: VALID-IN-CONTEXT — conditions under which the claim applies
    field :valid_in, :string
    field :valid_in_context, :map, default: %{}
    # Noepedia: coverage — relevant networks identified vs evaluated (coverage ≠ confidence)
    field :coverage_relevant, :integer, default: 0
    field :coverage_evaluated, :integer, default: 0
    field :coverage_not_yet, :integer, default: 0

    belongs_to :user, Web.Accounts.User
    has_many :evidence_items, Web.Knowledge.Evidence
    has_many :reviews, Web.Knowledge.Review

    timestamps()
  end

  def changeset(claim, attrs) do
    claim
    |> cast(attrs, [
      :subject,
      :predicate,
      :object,
      :status,
      :confidence,
      :domain,
      :user_id,
      :replication_count,
      :valid_in,
      :valid_in_context,
      :coverage_relevant,
      :coverage_evaluated,
      :coverage_not_yet
    ])
    |> validate_required([:subject, :predicate, :object, :user_id])
    |> validate_inclusion(:status, @statuses)
    |> validate_number(:replication_count, greater_than_or_equal_to: 0)
    |> validate_number(:coverage_relevant, greater_than_or_equal_to: 0)
    |> validate_number(:coverage_evaluated, greater_than_or_equal_to: 0)
    |> validate_number(:coverage_not_yet, greater_than_or_equal_to: 0)
  end
end
