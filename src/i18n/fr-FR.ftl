nav-add = Ajouter
nav-products = Produits
nav-plan = Où acheter
nav-settings = Réglages

add-title = Nouvel achat
field-product = Produit
field-store = Magasin
field-price = Prix payé
field-quantity = Quantité
field-unit = Unité
field-date = Date
field-note = Note
field-note-placeholder = Marque, promotion…
field-product-placeholder = Beurre doux
field-store-placeholder = Lidl
field-piece-weight = Poids d'une pièce (optionnel)
field-pack = Pack
field-pack-none = Quantité libre
field-pack-label = Nom du pack
field-pack-pieces = Nombre de pièces par pack

unit-kg = kilo
unit-g = gramme
unit-l = litre
unit-ml = millilitre
unit-piece = pièce

action-add = Ajouter
action-delete = Supprimer
action-cancel = Annuler
action-confirm = Confirmer
action-import = Importer
action-export = Exporter
action-back = Retour

error-product = Indique un produit.
error-store = Indique un magasin.
error-price = Prix invalide — écris par exemple 2,45.
error-quantity = Quantité invalide — elle doit être supérieure à zéro.
error-piece-weight = Poids invalide — laisse ce champ vide si tu ne le connais pas.
error-pack-label = Indique un nom pour ce pack.
error-pack-pieces = Nombre de pièces invalide — il doit être supérieur à zéro.

recent-title = Derniers ajouts
recent-empty = Aucun achat enregistré pour l'instant.

products-title = Produits
products-search = Rechercher un produit
products-store-all = Tous les magasins
products-empty = Ajoute un premier achat pour voir apparaître tes produits.
products-no-match = Aucun produit ne correspond.

label-best = Moins cher
label-purchases =
    { $count ->
        [one] { $count } achat
       *[other] { $count } achats
    }
label-since = depuis le { $date }

trend-up = En hausse
trend-down = En baisse
trend-flat = Stable
trend-new = Premier relevé

detail-not-found-title = Produit introuvable
detail-not-found = Ce produit n'existe plus.
detail-evolution = Évolution du prix unitaire
detail-by-store = Comparatif par magasin
detail-history = Historique
detail-single-point = Un seul relevé pour l'instant — ajoute un autre achat pour voir la courbe.
detail-mixed-units = Les achats dans une autre unité restent dans l'historique mais sont exclus de la comparaison.

plan-title = Où acheter quoi
plan-intro = Chaque produit sous le magasin qui le vend le moins cher en moyenne.
plan-empty = Enregistre un même produit dans deux magasins pour obtenir une comparaison.
plan-savings = { $amount } d'écart par unité

settings-title = Réglages
settings-language = Langue
settings-data = Données
settings-stats = { $purchases } achats · { $products } produits · { $stores } magasins
settings-packs = Mes packs
settings-packs-help = Définis une fois la taille d'un pack (ex. Coca-Cola par 6) pour la retrouver comme raccourci dans le formulaire d'ajout.
settings-packs-empty = Aucun pack défini pour l'instant.
settings-export = Sauvegarde
settings-backup-help = Enregistre tes achats dans un fichier JSON, à garder en lieu sûr.
settings-export-done = Sauvegarde exportée.
settings-export-error = Impossible d'enregistrer le fichier.
settings-export-fallback-help = Le partage n'a pas fonctionné sur cet appareil — copie ce texte à la place.
settings-import = Restauration
settings-restore-help = Choisis un fichier de sauvegarde JSON. Il remplacera tout l'historique actuel.
settings-import-error = Sauvegarde illisible — rien n'a été modifié.
settings-import-fallback-help = Le sélecteur de fichier n'a pas fonctionné sur cet appareil — colle une sauvegarde ici.
settings-import-done =
    { $count ->
        [one] { $count } achat restauré.
       *[other] { $count } achats restaurés.
    }
settings-wipe = Tout effacer
settings-wipe-warning = Tout l'historique sera définitivement supprimé.
settings-storage = Tout est stocké sur cet appareil uniquement. Aucune donnée ne circule sur le réseau.
