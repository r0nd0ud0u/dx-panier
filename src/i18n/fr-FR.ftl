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

error-product = Indique un produit.
error-store = Indique un magasin.
error-price = Prix invalide — écris par exemple 2,45.
error-quantity = Quantité invalide — elle doit être supérieure à zéro.

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
settings-export = Sauvegarde
settings-export-help = Copie ce texte et garde-le en lieu sûr.
settings-import = Restauration
settings-import-help = Colle ici une sauvegarde. Elle remplacera tout l'historique actuel.
settings-import-error = Sauvegarde illisible — rien n'a été modifié.
settings-import-done =
    { $count ->
        [one] { $count } achat restauré.
       *[other] { $count } achats restaurés.
    }
settings-wipe = Tout effacer
settings-wipe-warning = Tout l'historique sera définitivement supprimé.
settings-storage = Tout est stocké sur cet appareil uniquement. Aucune donnée ne circule sur le réseau.
